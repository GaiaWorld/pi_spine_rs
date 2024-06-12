// pub use bevy_ecs::{prelude::{ResMut, Res, Commands, Changed, With, Or}, system::{Command, EntityCommands}, query::WorldQuery};

pub use pi_world::world::*;
pub use pi_world::prelude::{*, App};
pub use pi_world::prelude::{SingleResMut as ResMut, SingleRes as Res};
pub use pi_world_macros::{Resource};
pub use pi_world::{
    editor::EntityEditor, filter::Changed, schedule_config::{IntoSystemConfigs, StageLabel}, world::Entity,
    query::*,
};

pub type Commands<'a> = EntityEditor<'a>;


pub trait Component: Bundle + 'static + Send + Sync  {
    
}

impl<T: Bundle + 'static + Send + Sync> Component for T {

}

pub struct EntityCommandTemp(Entity);

impl EntityCommandTemp {
    pub fn id(&self) -> Entity {
        self.0
    }
}

pub struct EntityCommands<'w, 'a> {
    entity: Entity,
    commands: &'a mut EntityEditor<'w>,
}

pub struct EntityCommandsEmpty<'w> {
    entity: Entity,
    commands: EntityEditor<'w>,
}
impl<'w> EntityCommandsEmpty<'w> {
    #[inline]
    #[must_use = "Omit the .id() call if you do not need to store the `Entity` identifier."]
    pub fn id(&self) -> Entity {
        self.entity
    }
}

impl<'w, 'a> EntityCommands<'w, 'a> {
    #[inline]
    #[must_use = "Omit the .id() call if you do not need to store the `Entity` identifier."]
    pub fn id(&self) -> Entity {
        self.entity
    }
    pub fn insert<A: Bundle + 'static>(&mut self, bundle: A) -> &mut Self {
        
        // let entity = self.entity;
        // let mut alter = self.alter::<A>();
        // match alter.alter(entity, bundle) {
        //     Ok(_) => {}
        //     Err(e) => {
        //         // log::error!(" insert {:?}", e);
        //     }
        // }
        
        match self.commands.add_components(self.entity, bundle) {
            Ok(_) => {}
            Err(e) => {
                // log::error!(" insert {:?}", e);
            }
        }

        self
    }
    // pub fn insert_alter<A: Bundle + 'static>(&mut self, bundle: A, mut alter: Option<pi_world::alter::Alterer<'a, (), (), A, ()>>) -> Option<pi_world::alter::Alterer<'a, (), (), A, ()>> {

    //     let entity = self.entity;
    //     if alter.is_none() {
    //         alter = Some(self.commands.world().make_alterer::<(), (), A, ()>());
    //     }
    //     match alter.as_mut().unwrap().alter(entity, bundle) {
    //         Ok(_) => {}
    //         Err(e) => { log::error!(" insert {:?}", e); }
    //     }
    //     alter
    // }
    // pub fn alter<A: Bundle + 'static>(&mut self) -> pi_world::alter::Alterer<(), (), A, ()> {
    //     // self.commands.world().make_alterer::<(), (), A, ()>()
    // }
    pub fn despawn(&mut self) {
        self.commands.destroy(self.entity);
    }
}


pub trait TEntityCommands<'w> {
    fn spawn_empty<'a>(&'a mut self) -> EntityCommands<'w, 'a>;
    fn spawn<'a, M: Bundle + 'static>(&'a mut self, bundle: M) -> EntityCommands<'w, 'a>;
    fn get_entity<'a>(&'a mut self, entity: Entity) -> Option<EntityCommands<'w, 'a>>;
    fn entity<'a>(&'a mut self, entity: Entity) -> EntityCommands<'w, 'a>;
}

impl<'w> TEntityCommands<'w> for EntityEditor<'w> {
    fn spawn_empty<'a>(&'a mut self) -> EntityCommands<'w, 'a> {
        
        let entity = self.insert_entity(()); // self.world().make_inserter().insert(());

        // let entity = self.alloc_entity();
        EntityCommands {
            entity: entity,
            commands: self
        }
    }
    fn spawn<'a, A: Bundle + 'static>(&'a mut self, bundle: A) -> EntityCommands<'w, 'a> {
        // let entity = self.world().make_inserter().insert(());
        // match self.world().make_alterer::<(), (), A, ()>().alter(entity, bundle) {
        //     Ok(_) => {},
        //     Err(e) => { 
        //         // log::error!("spawn {:?}", e);
        //     }
        // }

        // let entity = self.alloc_entity();
        // let mut components = A::components(vec![]);
        // let mut indexs = vec![];

        // components.drain(..).for_each(|item| {
        //     indexs.push(item.world_index);
        // });
        // self.insert_components_by_index(&indexs);
        // *self.get_component_mut::<A>(entity).unwrap() = bundle;

        let entity = self.insert_entity(bundle);

        EntityCommands {
            entity,
            commands: self
        }
    }
    fn get_entity<'a>(&'a mut self, entity: Entity) -> Option<EntityCommands<'w, 'a>> {
        if self.contains_entity(entity) {
            Some(EntityCommands { entity, commands: self })
        } else {
            None
        }
    }
    fn entity<'a>(&'a mut self, entity: Entity) -> EntityCommands<'w, 'a> {
        EntityCommands { entity, commands: self }
    }
}


pub trait Resource {

}

impl<T: 'static> Resource for T {

}


pub trait AppResourceTemp {
    fn insert_resource<T: 'static>(&mut self, resource: T) -> &mut Self;
    fn add_systems<M>(&mut self, stage_label: impl StageLabel, system: impl IntoSystemConfigs<M>) -> &mut Self;
    fn update(&mut self);
}

impl AppResourceTemp for App {
    fn insert_resource<T: 'static>(&mut self, resource: T) -> &mut Self {
        self.world.insert_single_res(resource);
        self
    }
    fn add_systems<M>(&mut self, stage_label: impl StageLabel, system: impl IntoSystemConfigs<M>) -> &mut Self {
        self.add_system(stage_label, system)
    }
    fn update(&mut self) {
        self.run()
    }
}

pub trait WorldResourceTemp {
    fn insert_resource<T: 'static>(&mut self, resource: T)  -> &mut Self;
    fn get_resource<T: 'static>(&self) -> Option<& T>;
    fn get_resource_mut<T: 'static>(&mut self) -> Option<& mut T>;
    fn contains_resource<T: 'static>(&self) -> bool;
    fn spawn_empty<'w>(&'w mut self) -> EntityCommandsEmpty<'w>;
}

impl WorldResourceTemp for World {
    fn insert_resource<T: 'static>(&mut self, resource: T)  -> &mut Self {
        self.insert_single_res(resource);
        self
    }

    fn get_resource<T: 'static>(& self) -> Option<& T> {
        match self.get_single_res::<T>() {
            Some(res) => Some(&**res),
            None => None
        }
    }

    fn get_resource_mut<T: 'static>(&mut self) -> Option<&mut T> {
        match self.get_single_res_mut::<T>() {
            Some(res) => Some(&mut **res),
            None => None
        }
    }

    fn contains_resource<T: 'static>(&self) -> bool {
        self.contains_resource::<T>()
    }
    fn spawn_empty<'w>(&'w mut self) -> EntityCommandsEmpty<'w> {
        let entity = self.make_inserter().insert(());
        let mut editor = self.make_entity_editor();
        EntityCommandsEmpty {
            entity: entity,
            commands: editor
        }
    }
}


pub fn add_systems(label: impl StageLabel, ) {
    
}
