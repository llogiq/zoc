// See LICENSE file for copyright and license details.

use std::collections::{HashMap, VecDeque};
use cgmath::{Matrix4, Matrix3};
use core::types::{ZInt, /*ZFloat*/};
use core::{MapPos};
use camera::Camera;
use geom;
use move_helper::{MoveHelper};
use context::{Context, texture_from_bytes};
use tactical_screen::{Mesh};
use text;
use ::{Vertex};

struct ShowTextCommand {
    pos: MapPos,
    text: String,
}

struct MapText {
    move_helper: MoveHelper,
    mesh: Mesh,
    pos: MapPos,
}

pub struct MapTextManager {
    commands: VecDeque<ShowTextCommand>,
    visible_labels_list: HashMap<ZInt, MapText>,
    // scale: ZFloat,
    last_label_id: ZInt, // TODO: think about better way of deleting old labels
}

impl MapTextManager {
    pub fn new(/*font_stash: &mut FontStash*/) -> Self {
        MapTextManager {
            commands: VecDeque::new(),
            visible_labels_list: HashMap::new(),
            // scale: 0.5 / font_stash.get_size(),
            last_label_id: 0,
        }
    }

    pub fn add_text<P: AsRef<MapPos>>(&mut self, pos: &P, text: &str) {
        self.commands.push_back(ShowTextCommand {
            pos: pos.as_ref().clone(),
            text: text.to_owned(),
        });
    }

    fn can_show_text_here<P: AsRef<MapPos>>(&self, pos: &P) -> bool {
        let min_progress = 0.3;
        for (_, map_text) in &self.visible_labels_list {
            let progress = map_text.move_helper.progress();
            if map_text.pos == *pos.as_ref() && progress < min_progress {
                return false;
            }
        }
        true
    }

    pub fn do_commands(&mut self, context: &mut Context) {
        let mut postponed_commands = Vec::new();
        while !self.commands.is_empty() {
            let command = self.commands.pop_front()
                .expect("MapTextManager: Can`t get next command");
            if !self.can_show_text_here(&command.pos) {
                postponed_commands.push(command);
                continue;
            }
            let from = geom::map_pos_to_world_pos(&command.pos);
            let mut to = from.clone();
            to.v.z += 2.0;
            let mesh = {
                let (w, h, texture_data) = text::text_to_texture(&context.font, 80.0, &command.text);
                let texture = texture_from_bytes(&mut context.factory, w, h, &texture_data);
                let scale_factor = 200.0; // TODO: take camera zoom into account
                let h_2 = (h as f32 / scale_factor) / 2.0;
                let w_2 = (w as f32 / scale_factor) / 2.0;
                let vertices = &[
                    Vertex{pos: [-w_2, -h_2, 0.0], uv: [0.0, 1.0]},
                    Vertex{pos: [-w_2, h_2, 0.0], uv: [0.0, 0.0]},
                    Vertex{pos: [w_2, -h_2, 0.0], uv: [1.0, 1.0]},
                    Vertex{pos: [w_2, h_2, 0.0], uv: [1.0, 0.0]},
                ];
                let indices: &[u16] = &[0,  1,  2,  1,  2,  3];
                let mesh = Mesh::new(context, vertices, indices, texture);
                mesh
            };
            self.visible_labels_list.insert(self.last_label_id, MapText {
                pos: command.pos.clone(),
                mesh: mesh,
                move_helper: MoveHelper::new(&from, &to, 1.0),
            });
            self.last_label_id += 1;
        }
        self.commands.extend(postponed_commands);
    }

    fn delete_old(&mut self) {
        let mut bad_keys = Vec::new();
        for (key, map_text) in &self.visible_labels_list {
            if map_text.move_helper.is_finished() {
                bad_keys.push(*key);
            }
        }
        for key in &bad_keys {
            self.visible_labels_list.remove(key);
        }
    }

    pub fn draw(
        &mut self,
        context: &mut Context,
        camera: &Camera,
        dtime: u64,
    ) {
        self.do_commands(context);
        // TODO: I'm not sure that disabling depth test is correct solution
        // context.zgl.set_depth_test(false);
        let rot_z_mat = Matrix4::from(Matrix3::from_angle_z(camera.get_z_angle()));
        let rot_x_mat = Matrix4::from(Matrix3::from_angle_x(camera.get_x_angle()));
        context.data.basic_color = [0.0, 0.0, 0.0, 1.0];
        for (_, map_text) in &mut self.visible_labels_list {
            let pos = map_text.move_helper.step(dtime);
            let tr_mat = Matrix4::from_translation(pos.v);
            let mvp = camera.mat() * tr_mat * rot_z_mat * rot_x_mat;
            context.data.mvp = mvp.into();
            context.data.texture.0 = map_text.mesh.texture.clone();
            context.data.vbuf = map_text.mesh.vertex_buffer.clone();
            context.encoder.draw(&map_text.mesh.slice, &context.pso, &context.data);
        }
        // context.zgl.set_depth_test(true);
        self.delete_old();
    }
}

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
