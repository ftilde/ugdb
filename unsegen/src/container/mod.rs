use base::{Window, GraphemeCluster};
use widget::{Widget, Demand2D, RenderingHints};
use widget::layouts::{HorizontalLayout, VerticalLayout, SeparatingStyle};
use input::{Behavior, Input, Navigatable, NavigateBehavior, OperationResult};
use std::collections::BTreeMap;

pub trait Container<P: ?Sized> : Widget {
    fn input(&mut self, input: Input, parameters: &mut P) -> Option<Input>;
}

pub struct Accessor<T: ?Sized + ContainerProvider> {
    pub access: fn(&T) -> &Container<T::Parameters>,
    pub access_mut: fn(&mut T) -> &mut Container<T::Parameters>,
}

pub trait ContainerProvider {
    type Parameters;
    fn get_accessor(identifier: &str) -> Option<Accessor<Self>>;
    const DEFAULT_CONTAINER: &'static str;
}

struct ContainerBehavior<'a, 'b, P: 'a + 'b> {
    container: &'a mut Container<P>,
    parameters: &'b mut P,
}

impl<'a, 'b, P> Behavior for ContainerBehavior<'a, 'b, P> {
    fn input(self, i: Input) -> Option<Input> {
        self.container.input(i, self.parameters)
    }
}

pub enum LayoutNode {
    HorizontalSplit(Vec<LayoutNode>),
    VerticalSplit(Vec<LayoutNode>),
    Container(String),
}

impl LayoutNode {
    fn visit_containers<E, F: FnMut(&str) -> Result<(), E>>(&self, f: &mut F) -> Result<(), E> {
        match self {
            &LayoutNode::Container(ref name) => {
                f(name)?;
            },
            &LayoutNode::VerticalSplit(ref nodes) => {
                for node in nodes {
                    node.visit_containers(f)?;
                }
            },
            &LayoutNode::HorizontalSplit(ref nodes) => {
                for node in nodes {
                    node.visit_containers(f)?;
                }
            },
        }
        Ok(())
    }
}

struct RenderNode<'a, 'b, C: ContainerProvider + 'a> {
    layout: &'b LayoutNode,
    app: &'a ApplicationAccessWrapper<'a, 'a, C>,
}

impl<'a, 'b, C: ContainerProvider + 'a> Widget for RenderNode<'a, 'b, C> {
    fn space_demand(&self) -> Demand2D {
        match self.layout {
            &LayoutNode::Container(ref name) => {
                self.app.get_container(name).space_demand()
            }
            &LayoutNode::VerticalSplit(ref nodes) => {
                let layout = VerticalLayout::new(SeparatingStyle::Draw(GraphemeCluster::try_from('=').unwrap()));
                let nodes: Vec<_> = nodes.iter().map(|n| RenderNode {layout: n, app: self.app } ).collect();
                let node_refs: Vec<&Widget> = nodes.iter().map(|n| n as &Widget).collect();
                layout.space_demand(node_refs.as_slice())
            }
            &LayoutNode::HorizontalSplit(ref nodes) => {
                let layout = HorizontalLayout::new(SeparatingStyle::Draw(GraphemeCluster::try_from('|').unwrap()));
                let nodes: Vec<_> = nodes.iter().map(|n| RenderNode {layout: n, app: self.app } ).collect();
                let node_refs: Vec<&Widget> = nodes.iter().map(|n| n as &Widget).collect();
                layout.space_demand(node_refs.as_slice())
            }
        }
    }
    fn draw(&self, window: Window, hints: RenderingHints) {
        match self.layout {
            &LayoutNode::Container(ref name) => {
                let hints = if name == self.app.state.active_container.as_str() {
                    RenderingHints { active: true, ..hints }
                } else {
                    RenderingHints { active: false, ..hints }
                };
                self.app.get_container(name).draw(window, hints)
            }
            &LayoutNode::VerticalSplit(ref nodes) => {
                let layout = VerticalLayout::new(SeparatingStyle::Draw(GraphemeCluster::try_from('=').unwrap()));
                let nodes: Vec<_> = nodes.iter().map(|n| RenderNode { layout: n, app: self.app } ).collect();
                let node_refs: Vec<(&Widget, RenderingHints)> = nodes.iter().map(|n| (n as &Widget, hints)).collect();
                layout.draw(window, node_refs.as_slice())
            }
            &LayoutNode::HorizontalSplit(ref nodes) => {
                let layout = HorizontalLayout::new(SeparatingStyle::Draw(GraphemeCluster::try_from('|').unwrap()));
                let nodes: Vec<_> = nodes.iter().map(|n| RenderNode { layout: n, app: self.app } ).collect();
                let node_refs: Vec<(&Widget, RenderingHints)> = nodes.iter().map(|n| (n as &Widget, hints)).collect();
                layout.draw(window, node_refs.as_slice())
            }
        }
    }
}


struct ApplicationAccessWrapper<'a, 'b, C: ContainerProvider + 'a + 'b> {
    container_provider: &'a mut C,
    state: &'b ApplicationState<C>,
}

impl<'a, 'b, C: ContainerProvider + 'a> ApplicationAccessWrapper<'a, 'b, C> {
    fn get_container<S: AsRef<str>>(&self, name: S) -> &Container<C::Parameters> {
        (self.state.accessor_cache[name.as_ref()].access)(self.container_provider)
    }

    /*
    fn get_container_mut<S: AsRef<str>>(&mut self, name: S) -> &mut Container<Parameters=C::Parameters> {
        (self.state.accessor_cache[name.as_ref()].access_mut)(self.container_provider)
    }
    */

    fn get_active_container_mut(&mut self) -> &mut Container<C::Parameters> {
        (self.state.accessor_cache[self.state.active_container.as_str()].access_mut)(self.container_provider)
    }
}

impl<'a, 'b, C: ContainerProvider + 'a> Navigatable for ApplicationAccessWrapper<'a, 'b, C> {
    fn move_up(&mut self) -> OperationResult {
        Ok(())
    }
    fn move_down(&mut self) -> OperationResult {
        Ok(())
    }
    fn move_left(&mut self) -> OperationResult {
        Ok(())
    }
    fn move_right(&mut self) -> OperationResult {
        Ok(())
    }
}

type AccessorMap<C> = BTreeMap<String, Accessor<C>>;

struct ApplicationState<C: ContainerProvider> {
    accessor_cache: AccessorMap<C>,
    active_container: String,
}

impl<C: ContainerProvider> ApplicationState<C> {
    fn from_layout_tree(layout_root: &LayoutNode) -> Result<Self, String> {
        let mut accessor_cache = AccessorMap::<C>::new();
        {
            let mut visit_func = |s: &str| -> Result<(), String> {
                let accessor = C::get_accessor(s).ok_or(s)?;
                accessor_cache.insert(s.to_owned(), accessor);
                Ok(())
            };
            layout_root.visit_containers(&mut visit_func)?;
        }
        Ok(ApplicationState {
            accessor_cache: accessor_cache,
            active_container: C::DEFAULT_CONTAINER.to_owned(),
        })
    }
}

pub struct Application<C: ContainerProvider> {
    layout: LayoutNode,
    state: ApplicationState<C>,
}

impl<C: ContainerProvider> Application<C> {
    pub fn from_layout_tree(layout_root: LayoutNode) -> Result<Self, String> {
        let state = ApplicationState::from_layout_tree(&layout_root)?;
        Ok(Application {
            layout: layout_root,
            state: state,
        })
    }

    fn build_wrapper<'a, 'b>(&'b self, provider: &'a mut C) -> ApplicationAccessWrapper<'a, 'b, C> {
        ApplicationAccessWrapper {
            container_provider: provider,
            state: &self.state,
        }
    }

    pub fn draw(&self, window: Window, provider: &mut C) {
        let wrapper = ApplicationAccessWrapper {
            container_provider: provider,
            state: &self.state,
        };
        RenderNode { layout: &self.layout, app: &wrapper }.draw(window, RenderingHints::default());
    }
}

pub struct ApplicationBehavior<'a, 'b, 'c, C: ContainerProvider + 'a + 'b>
where C::Parameters: 'c
{
    app: &'a mut Application<C>,
    provider: &'b mut C,
    parameters: &'c mut C::Parameters,
}

impl<'a, 'b, 'c, C: ContainerProvider + 'a + 'b> ApplicationBehavior<'a, 'b, 'c, C> {
    pub fn new(app: &'a mut Application<C>, provider: &'b mut C, parameters: &'c mut C::Parameters) -> Self {
        ApplicationBehavior {
            app: app,
            provider: provider,
            parameters: parameters,
        }
    }
}

impl<'a, 'b, 'c, C: ContainerProvider + 'a + 'b> Behavior for ApplicationBehavior<'a, 'b, 'c, C> {
    fn input(self, i: Input) -> Option<Input> {
        i.chain(ContainerBehavior {
            container: self.app.build_wrapper(self.provider).get_active_container_mut(),
            parameters: self.parameters,
        }).chain(NavigateBehavior::new( &mut self.app.build_wrapper(self.provider) ))
        .finish()
    }
}
