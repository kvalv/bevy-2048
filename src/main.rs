use bevy::core::FixedTimestep;
use std::collections::hash_set::HashSet;
use bevy::prelude::*;
use itertools::Itertools;
use rand::prelude::*;
use rand::thread_rng;
use rand::seq::SliceRandom;
use std::collections::HashMap;

const WIDTH: u32 = 4;
const HEIGHT: u32 = 4;

#[derive(Default)]
struct Block {
    value: u32,
    merged: bool,
}
#[derive(PartialEq, Clone, Copy, Debug)]
struct Position {
    x: u32,
    y: u32,
}
struct Size {
    x: f32,
    y: f32,
}
#[derive(PartialEq, Clone, Copy, Debug)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}
struct SwipeEvent(Direction);
struct BlockColor {
    block0: Handle<ColorMaterial>,
    block1: Handle<ColorMaterial>,
    block2: Handle<ColorMaterial>,
    block3: Handle<ColorMaterial>,
    block4: Handle<ColorMaterial>,
    block5: Handle<ColorMaterial>,
    block6: Handle<ColorMaterial>,
    block7: Handle<ColorMaterial>,
    block8: Handle<ColorMaterial>,
    block9: Handle<ColorMaterial>,
}

fn block_scale(win: Res<Windows>, mut q: Query<(&Size, &mut Sprite), With<Block>>) {
    let win_width = win.get_primary().unwrap().width() as f32;
    let win_height = win.get_primary().unwrap().height() as f32;
    for (sz, mut sprite) in q.iter_mut() {
        let sx = (win_width / WIDTH as f32) * sz.x;
        let sy = (win_height / WIDTH as f32) * sz.y;
        sprite.size = Vec2::new(sx, sy);
    }
}

fn block_pos(win: Res<Windows>, mut q: Query<(&mut Transform, &Position), With<Block>>) {
    let w = win.get_primary().unwrap().width() as f32;
    let h = win.get_primary().unwrap().height() as f32;
    for (mut tx, pos) in q.iter_mut() {
        let x = (pos.x as f32 / WIDTH as f32) * w - w / 2.0 + (w / WIDTH as f32) / 2.0;
        let y = (pos.y as f32 / WIDTH as f32) * h - h / 2.0 + (h / HEIGHT as f32) / 2.0;
        tx.translation = Vec3::new(x, y, 0.0);
    }
}

fn input_events(kbd: Res<Input<KeyCode>>, mut writer: EventWriter<SwipeEvent>) {
    if kbd.just_pressed(KeyCode::W) {
        writer.send(SwipeEvent(Direction::Up))
    } else if kbd.just_pressed(KeyCode::A) {
        writer.send(SwipeEvent(Direction::Left));
    } else if kbd.just_pressed(KeyCode::S) {
        writer.send(SwipeEvent(Direction::Down));
    } else if kbd.just_pressed(KeyCode::D) {
        writer.send(SwipeEvent(Direction::Right));
    }
}

fn block_color(mut q: Query<(&Block, &mut Handle<ColorMaterial>)>, materials: Res<BlockColor>) {
    for (
        Block {
            value: v,
            merged: _,
        },
        mut handle,
    ) in q.iter_mut()
    {
        let col = match *v {
            0 => materials.block0.clone(),
            1 => materials.block1.clone(),
            2 => materials.block2.clone(),
            3 => materials.block3.clone(),
            4 => materials.block4.clone(),
            5 => materials.block5.clone(),
            6 => materials.block6.clone(),
            7 => materials.block7.clone(),
            8 => materials.block8.clone(),
            9 => materials.block9.clone(),
            _ => materials.block9.clone(),
        };
        *handle = col;
    }
}

fn block_swipe(
    mut cmd: Commands,
    mut reader: EventReader<SwipeEvent>,
    mut q: Query<(&mut Block, &mut Position, Entity)>,
) {
    for ev in reader.iter() {
        //let m: HashMap<u32, u32>  = q.iter().map(|x| (x.x, x.x));
        //let rows: HashMap<_, _> = q.iter_mut().map(|x| (x.y, x)).collect();

        if ev.0 == Direction::Up || ev.0 == Direction::Down {
            let mut cols: HashMap<u32, Vec<_>> = HashMap::new();
            for e in q.iter_mut() {
                cols.entry(e.1.x).or_insert(Vec::new()).push(e);
            }
            for c in 0u32..WIDTH {
                if let Some(v) = cols.get_mut(&c) {
                    if ev.0 == Direction::Up {
                        v.sort_by(|b, a| a.1.x.cmp(&b.1.x));
                    } else {
                        v.sort_by(|a, b| a.1.x.cmp(&b.1.x));
                    }
                    // bottom to top...
                    for idx in 0..v.len() {
                        if idx == 0 {
                            if ev.0 == Direction::Down {
                                v[0].1.y = 0;
                            } else {
                                v[0].1.y = HEIGHT - 1;
                            }
                        } else {
                            let (left, right) = v.split_at_mut(idx);
                            let (other_block, _, _) = left.last_mut().unwrap();
                            if let Some((block, pos, ent)) = right.get_mut(0) {
                                if block.value == other_block.value && !other_block.merged {
                                    other_block.merged = true;
                                    other_block.value += 1;
                                    cmd.entity(*ent).despawn();
                                } else {
                                    pos.y = idx as u32;
                                }
                            }
                        }
                    }
                }
            }
        } else {
            let mut rows: HashMap<u32, Vec<_>> = HashMap::new();
            for e in q.iter_mut() {
                rows.entry(e.1.y).or_insert(Vec::new()).push(e);
            }
            for r in 0u32..HEIGHT {
                if let Some(v) = rows.get_mut(&r) {
                    if ev.0 == Direction::Right {
                        v.sort_by(|b, a| a.1.x.cmp(&b.1.x));
                    } else {
                        v.sort_by(|a, b| a.1.x.cmp(&b.1.x));
                    }
                    // bottom to top...
                    for idx in 0..v.len() {
                        if idx == 0 {
                            if ev.0 == Direction::Left {
                                v[0].1.x = 0;
                            } else {
                                v[0].1.x = HEIGHT - 1;
                            }
                        } else {
                            let (left, right) = v.split_at_mut(idx);
                            let (other_block, _, _) = left.last_mut().unwrap();
                            if let Some((block, pos, ent)) = right.get_mut(0) {
                                if block.value == other_block.value && !other_block.merged {
                                    other_block.merged = true;
                                    other_block.value += 1;
                                    cmd.entity(*ent).despawn();
                                } else {
                                    pos.x = idx as u32;
                                }
                            }
                        }
                    }
                }
            }
        }

        for (mut block, _, _) in q.iter_mut() {
            block.merged = false;
        }
    }
}
fn setup(mut cmd: Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    cmd.spawn_bundle(OrthographicCameraBundle::new_2d());
    cmd.insert_resource(BlockColor {
        block0: materials.add(Color::rgb(0.7, 0.1, 0.1).into()),
        block1: materials.add(Color::rgb(0.5, 0.7, 0.1).into()),
        block2: materials.add(Color::rgb(0.1, 0.1, 0.7).into()),
        block3: materials.add(Color::rgb(0.1, 0.3, 0.3).into()),
        block4: materials.add(Color::rgb(0.3, 0.3, 0.1).into()),
        block5: materials.add(Color::rgb(0.3, 0.1, 0.3).into()),
        block6: materials.add(Color::rgb(0.8, 0.8, 0.1).into()),
        block7: materials.add(Color::rgb(0.1, 0.8, 0.8).into()),
        block8: materials.add(Color::rgb(0.8, 0.1, 0.8).into()),
        block9: materials.add(Color::rgb(0.4, 0.9, 0.1).into()),
    });
}

fn block_spawn(
    mut cmd: Commands,
    mut ev: EventReader<SwipeEvent>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    q: Query<&Position, With<Block>>
) {
    if ev.iter().next().is_none() {
        return;
    }
    let all_choices = (0u32..HEIGHT).cartesian_product(0u32..WIDTH).collect::<HashSet<_>>();
    let ex_choices = q.iter().map(|pos| (pos.x, pos.y)).collect::<HashSet<_>>();
    let rest = all_choices.difference(&ex_choices).collect::<Vec<_>>();

    let idx = rand::random::<usize>() % rest.len();
    let (x, y) = rest[idx];

    let v = rand::random::<u32>() % 1;
    cmd.spawn_bundle(SpriteBundle {
        material: materials.add(Color::rgb(0.9, 0.9, 0.9).into()),
        sprite: Sprite::new(Vec2::new(10.0, 10.0)),
        ..Default::default()
    })
    .insert(Block {
        value: v,
        merged: false,
    })
    .insert(Size { x: 0.8, y: 0.8 })
    .insert(Position { x: *x, y: *y })
    .id();
}

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_event::<SwipeEvent>()
        .insert_resource(WindowDescriptor {
            title: "2048".to_string(),
            width: 500.0,
            height: 500.0,
            ..Default::default()
        })
        .add_system(block_swipe.system())
        .insert_resource(ClearColor(Color::rgb(0.1, 0.1, 0.15)))
        .add_system_set_to_stage(
            CoreStage::PostUpdate,
            SystemSet::new()
                .with_system(block_pos.system())
                .with_system(block_scale.system()),
        )
        .add_system(block_spawn.system())
        .add_startup_system(setup.system())
        .add_system(block_color.system())
        .add_system_set(SystemSet::new().with_run_criteria(FixedTimestep::step(1.0)))
        .add_system(input_events.system())
        .run();
}
