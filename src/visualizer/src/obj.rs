// See LICENSE file for copyright and license details.

use std::collections::{HashMap};
use std::fmt::{Debug};
use std::io::{BufRead};
use std::path::{Path};
use std::str::{SplitWhitespace, Split, FromStr};
use cgmath::{Vector3, Vector2};
use core::types::{ZFloat};
use core::fs;
use types::{VertexCoord, TextureCoord};
use ::{Vertex};

struct Line {
    vertex: [u16; 2],
}

type Face = [[u16; 3]; 3];

pub struct Model {
    coords: Vec<VertexCoord>,
    uvs: Vec<TextureCoord>,
    faces: Vec<Face>,
    lines: Vec<Line>,
}

fn parse_word<T: FromStr>(words: &mut SplitWhitespace) -> T
    where T::Err: Debug
{
    let str = words.next().expect("Can not read next word");
    str.parse().expect("Can not parse word")
}

fn parse_charsplit<T: FromStr>(words: &mut Split<char>) -> T
    where T::Err: Debug
{
    let str = words.next().expect("Can not read next word");
    str.parse().expect("Can not parse word")
}

impl Model {
    pub fn new<P: AsRef<Path>>(path: P) -> Model {
        let mut obj = Model {
            coords: Vec::new(),
            uvs: Vec::new(),
            faces: Vec::new(),
            lines: Vec::new(),
        };
        obj.read(path);
        obj
    }

    fn read_v(words: &mut SplitWhitespace) -> VertexCoord {
        VertexCoord{v: Vector3 {
            x: parse_word(words),
            // y: parse_word(words), // TODO: flip models
            y: -parse_word::<ZFloat>(words),
            z: parse_word(words),
        }}
    }

    fn read_vt(words: &mut SplitWhitespace) -> TextureCoord {
        TextureCoord{v: Vector2 {
            x: parse_word(words),
            y: 1.0 - parse_word::<ZFloat>(words), // flip
        }}
    }

    fn read_f(words: &mut SplitWhitespace) -> [[u16; 3]; 3] {
        let mut f = [[0; 3]; 3];
        for (i, group) in words.by_ref().enumerate() {
            let w = &mut group.split('/');
            f[i] = [
                parse_charsplit(w),
                parse_charsplit(w),
                parse_charsplit(w),
            ];
        }
        f
    }

    fn read_l(words: &mut SplitWhitespace) -> Line {
        Line {
            vertex: [
                parse_word(words),
                parse_word(words),
            ],
        }
    }

    fn read_line(&mut self, line: &str) {
        let mut words = line.split_whitespace();
        fn is_correct_tag(tag: &str) -> bool {
            tag.len() != 0 && !tag.starts_with("#")
        }
        match words.next() {
            Some(tag) if is_correct_tag(tag) => {
                let w = &mut words;
                match tag {
                    "v" => self.coords.push(Model::read_v(w)),
                    "vn" => {},
                    "vt" => self.uvs.push(Model::read_vt(w)),
                    "f" => self.faces.push(Model::read_f(w)),
                    "l" => self.lines.push(Model::read_l(w)),
                    "s" => {},
                    "#" => {},
                    unexpected_tag => {
                        println!("obj: unexpected tag: {}", unexpected_tag);
                    }
                }
            }
            _ => {},
        };
    }

    fn read<P: AsRef<Path>>(&mut self, path: P) {
        for line in fs::load(path).lines() {
            match line {
                Ok(line) => self.read_line(&line),
                Err(msg) => panic!("Obj: read error: {}", msg),
            }
        }
    }

    pub fn is_wire(&self) -> bool {
        !self.lines.is_empty()
    }
}

// TODO: упростить
pub fn build(model: Model) -> (Vec<Vertex>, Vec<u16>) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let mut h: HashMap<(u16, u16), u16> = HashMap::new();
    for f in model.faces {
        for v in &f {
            let pos_id = v[0] - 1;
            let uv_id = v[1] - 1;
            let key = (pos_id, uv_id);
            let id = if h.contains_key(&key) {
                *h.get(&key).unwrap()
            } else {
                let id = vertices.len() as u16;
                vertices.push(Vertex {
                    pos: model.coords[pos_id as usize].v.into(),
                    uv: model.uvs[uv_id as usize].v.into(),
                });
                h.insert(key, id);
                id
            };
            indices.push(id);
        }
    }
    for line in model.lines {
        for i in 0 .. line.vertex.len() {
            let vertex_id = line.vertex[i] as usize - 1;
            vertices.push(Vertex {
                pos: model.coords[vertex_id].v.into(),
                uv: [0.0, 0.0],
            });
            indices.push(vertex_id as u16);
        }
    }
    (vertices, indices)
}

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
