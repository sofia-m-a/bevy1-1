use bevy::{prelude::*, reflect::TypeUuid};

// Create the animation asset
#[derive(Clone, Component)]
pub struct Animation(pub Handle<AnimationAsset>);

#[derive(TypeUuid, Deref)]
#[uuid = "ae6a74db-f6fa-43c4-ac16-01d13b50e4c6"]
pub struct AnimationAsset(pub benimator::Animation);

// Create the player component
#[derive(Default, Component, Deref, DerefMut)]
pub struct AnimationState(pub benimator::State);

fn animate(
    time: Res<Time>,
    animations: Res<Assets<AnimationAsset>>,
    mut query: Query<(&mut AnimationState, &mut TextureAtlasSprite, &Animation)>,
) {
    for (mut player, mut texture, animation) in query.iter_mut() {
        // Update the state
        if let Some(a) = animations.get(&animation.0) {
            player.update(a, time.delta());
        }

        // Update the texture atlas
        texture.index = player.frame_index();
    }
}

#[derive(Clone, Copy, Debug)]
pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(CoreStage::PostUpdate, animate);
    }
}
