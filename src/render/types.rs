use std::ffi::NulError;
use std::fmt::{Formatter, Write};
use std::string::FromUtf8Error;
use image::ImageError;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::{Error, MapAccess, SeqAccess, Visitor};
use serde::ser::SerializeStruct;
use thiserror::Error;

pub(crate) type Vec2f = [f32; 2];
pub type Vec3f = [f32; 3];
type Vec4f = [f32; 4];
type Vec2d = [f64; 2];
type Vec3d = [f64; 3];
type Vec4d = [f64; 4];
type Vec2i = [i32; 2];
type Vec3i = [i32; 3];
type Vec4i = [i32; 4];
type Vec2l = [i64; 2];
type Vec3l = [i64; 3];
type Vec4l = [i64; 4];
type Vec2ui = [u32; 2];
type Vec3ui = [u32; 3];
type Vec4ui = [u32; 4];
type Vec2ul = [u64; 2];
type Vec3ul = [u64; 3];
type Vec4ul = [u64; 4];
pub type Vec2ub = [u8; 2];
pub type Vec3ub = [u8; 3];
type Vec4ub = [u8; 4];
type Vec2b = [i8; 2];
pub type Vec3b = [i8; 3];
type Vec4b = [i8; 4];
type Vec2us = [u16; 2];
pub type Vec3us = [u16; 3];
type Vec4us = [u16; 4];
type Vec2s = [i16; 2];
pub type Vec3s = [i16; 3];
type Vec4s = [i16; 4];
type RGBColor = [f32; 3];
type ARGBColor = [f32; 4];

type TexCoord = Vec2f;


#[derive(Clone)]
#[repr(C, packed)]
pub struct Vertex(pub Vec3f, pub TexCoord);

impl Serialize for Vertex {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        let mut state = serializer.serialize_struct("vertex", 2)?;
        let pos = self.0;
        let tex = self.1;
        state.serialize_field("pos", &pos)?;
        state.serialize_field("tex", &tex)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for Vertex {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        enum Field { Pos, Tex }

        impl<'de> Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {

                struct FieldVisitor;

                impl<'de> Visitor<'de> for FieldVisitor {
                    type Value = Field;

                    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                        formatter.write_str("`pos` or `tex`")
                    }

                    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E> where E: Error {
                        match v {
                            "pos" => Ok(Field::Pos),
                            "tex" => Ok(Field::Tex),
                            _ => Err(Error::unknown_field(v, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct VertexVisitor;

        impl<'de> Visitor<'de> for VertexVisitor {
            type Value = Vertex;

            fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                formatter.write_str("struct Vertex")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error> where A: SeqAccess<'de> {
                let pos = seq.next_element()?
                    .ok_or_else(|| Error::invalid_length(0, &self))?;
                let tex = seq.next_element()?
                    .ok_or_else(|| Error::invalid_length(1, &self))?;
                Ok(Vertex(pos, tex))
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error> where A: MapAccess<'de> {
                let mut pos = None;
                let mut tex = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Pos => {
                            if pos.is_some() {
                                return Err(Error::duplicate_field("pos"));
                            }
                            pos = Some(map.next_value()?);
                        }
                        Field::Tex => {
                            if tex.is_some() {
                                return Err(Error::duplicate_field("tex"));
                            }
                            tex = Some(map.next_value()?);
                        }
                    }
                }

                let pos = pos.ok_or_else(|| Error::missing_field("pos"))?;
                let tex = tex.ok_or_else(|| Error::missing_field("tex"))?;
                Ok(Vertex(pos, tex))
            }
        }

        const FIELDS: &'static [&'static str] = &["pos", "tex"];

        deserializer.deserialize_struct("Vertex", FIELDS, VertexVisitor)
    }
}

#[derive(Clone, Debug)]
#[repr(C, packed)]
pub struct Mat4f {
    pub matrix: [[f32; 4]; 4],
}

//Matrix column -> data_row[i]
/*
M = (a b c d
     e f g h
     i j k l
     m n o p)

matrix[0] = [a, e, i, m];
matrix[1] = [b, f, j, n];
matrix[2] = [c, g, k, o];
matrix[3] = [d, h, l, p];
 */

impl Mat4f {
    pub fn new() -> Self {
        Self {
            matrix: [
                [0.0, 0.0, 0.0, 0.0,],
                [0.0, 0.0, 0.0, 0.0,],
                [0.0, 0.0, 0.0, 0.0,],
                [0.0, 0.0, 0.0, 0.0,],
            ],
        }
    }

    pub fn from_array(data: [[f32; 4]; 4]) -> Self {
        Self {
            matrix: data,
        }
    }

    pub fn perspective(&mut self, fov: f32, aspect_ratio: f32, z_near: f32, z_far: f32) -> & mut Self {
        /*let zm: f32 = (z_far - z_near);
        let zp: f32 = (z_far + z_near);
        let tan: f32 = (1.0 / ((fov / 2.0).tan()));

        self.matrix = [
            [(tan / aspect_ratio), 0.0, 0.0, 0.0],
            [0.0, tan, 0.0, 0.0],
            [0.0, 0.0, -(zp / zm), -1.0],
            [0.0, 0.0, -((2.0 * z_far * z_near) / zm), 0.0],
        ];*/
        let mut arg_mat = self.clone();

        let zm: f32 = (z_far - z_near);
        let zp: f32 = (z_far + z_near);
        let tan: f32 = (1.0 / ((fov / 2.0).tan()));

        //h = tan * z_near
        //w = h * aspect_ratio

        //pm - perspective matrix
        let pm00 = tan / aspect_ratio; //z_near / w
        let pm11 = tan; //z_near / h
        let pm22 = -(zp / zm);
        let pm32 = (-2.0 * z_far * z_near) / zm;

        //tm - temporary matrix
        let tm20 = (arg_mat.matrix[2][0] * pm22) - arg_mat.matrix[3][0];
        let tm21 = (arg_mat.matrix[2][1] * pm22) - arg_mat.matrix[3][1];
        let tm22 = (arg_mat.matrix[2][2] * pm22) - arg_mat.matrix[3][2];
        let tm23 = (arg_mat.matrix[2][3] * pm22) - arg_mat.matrix[3][3];

        self.matrix[0][0] = arg_mat.matrix[0][0] * pm00;
        self.matrix[0][1] = arg_mat.matrix[0][1] * pm00;
        self.matrix[0][2] = arg_mat.matrix[0][2] * pm00;
        self.matrix[0][3] = arg_mat.matrix[0][3] * pm00;

        self.matrix[1][0] = arg_mat.matrix[1][0] * pm11;
        self.matrix[1][1] = arg_mat.matrix[1][1] * pm11;
        self.matrix[1][2] = arg_mat.matrix[1][2] * pm11;
        self.matrix[1][3] = arg_mat.matrix[1][3] * pm11;

        self.matrix[3][0] = arg_mat.matrix[2][0] * pm32;
        self.matrix[3][1] = arg_mat.matrix[2][1] * pm32;
        self.matrix[3][2] = arg_mat.matrix[2][2] * pm32;
        self.matrix[3][3] = arg_mat.matrix[2][3] * pm32;

        self.matrix[2][0] = tm20;
        self.matrix[2][1] = tm21;
        self.matrix[2][2] = tm22;
        self.matrix[2][3] = tm23;

        self
    }

    pub fn identity(&mut self) -> & mut Self {
        self.matrix = [
            [1.0, 0.0, 0.0, 0.0,],
            [0.0, 1.0, 0.0, 0.0,],
            [0.0, 0.0, 1.0, 0.0,],
            [0.0, 0.0, 0.0, 1.0,],
        ];
        self
    }

    pub fn get_rotation_matrix_axis_x(angle: f32) -> Self {
        let cos: f32 = angle.cos();
        let sin: f32 = angle.sin();
        Self {
            matrix: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, cos, sin, 0.0],
                [0.0, -sin, cos, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }
    pub fn get_rotation_matrix_axis_y(angle: f32) -> Self {
        let cos: f32 = angle.cos();
        let sin: f32 = angle.sin();
        Self {
            matrix: [
                [cos, 0.0, -sin, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [sin, 0.0, cos, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }
    pub fn get_rotation_matrix_axis_z(angle: f32) -> Self {
        let cos: f32 = angle.cos();
        let sin: f32 = angle.sin();
        Self {
            matrix: [
                [cos, sin, 0.0, 0.0],
                [-sin, cos, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    pub fn get_rotation_matrix_xyz(x: f32, y: f32, z: f32) -> Self {
        //TODO ROTATE WITH N: VEC3F AND ANGLE
        let cos_x: f32 = x.cos();
        let sin_x: f32 = x.sin();
        let cos_y: f32 = y.cos();
        let sin_y: f32 = y.sin();
        let cos_z: f32 = z.cos();
        let sin_z: f32 = z.sin();
        Self {
            matrix: [
                [(cos_y * cos_z),       ((sin_x * sin_y * cos_z) + (sin_z * cos_x)),     ((sin_x * sin_z) - (sin_y * cos_x * cos_z)),   0.0],
                [-(sin_z * cos_y),      ((cos_x * cos_z) - (sin_x * sin_y * sin_z)),     ((sin_x * cos_z) + (sin_y * sin_z * cos_x)),   0.0],
                [sin_y,                 -(sin_x * cos_y),                                (cos_x * cos_y),                               0.0],
                [0.0,                   0.0,                                             0.0,                                           1.0],
            ],
        }
        //TODO OPTIMIZED AND THEN N ANGLE ROTATION CAMERA MOVING
    }

    pub fn get_translation_matrix_xyz(x: f32, y: f32, z: f32) -> Self {
        Self {
            matrix: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [x,   y,   z,   1.0],
            ],
        }
    }

    pub fn get_scaling_matrix_xyz(sx: f32, sy: f32, sz: f32) -> Self {
        Self {
            matrix: [
                [sx,  0.0, 0.0, 0.0],
                [0.0, sy,  0.0, 0.0],
                [0.0, 0.0, sz,  0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    pub fn translate_xyz(&mut self, x: f32, y: f32, z: f32) -> &mut Self {
        //self.mul(&Self::get_translated_xyz(x, y, z));
        self.matrix[3][0] = ((self.matrix[0][0] * x) + (self.matrix[1][0] * y) + (self.matrix[2][0] * z) + self.matrix[3][0]);
        self.matrix[3][1] = ((self.matrix[0][1] * x) + (self.matrix[1][1] * y) + (self.matrix[2][1] * z) + self.matrix[3][1]);
        self.matrix[3][2] = ((self.matrix[0][2] * x) + (self.matrix[1][2] * y) + (self.matrix[2][2] * z) + self.matrix[3][2]);
        self.matrix[3][3] = ((self.matrix[0][3] * x) + (self.matrix[1][3] * y) + (self.matrix[2][3] * z) + self.matrix[3][3]);
        self
    }

    pub fn translate(&mut self, offset: &Vec3f) -> &mut Self {
        self.translate_xyz(offset[0], offset[1], offset[2])
    }

    pub fn rotate_x(&mut self, angle: f32) -> &mut Self {
        //self.mul(&Self::get_rotation_matrix_axis_x(angle));
        let cos: f32 = angle.cos();
        let sin: f32 = angle.sin();

        //rm - rotation matrix
        let rm11 = cos;
        let rm12 = sin;
        let rm21 = -sin;
        let rm22 = cos;

        //tm - temporary matrix
        let tm10 = (self.matrix[1][0] * rm11) + (self.matrix[2][0] * rm12);
        let tm11 = (self.matrix[1][1] * rm11) + (self.matrix[2][1] * rm12);
        let tm12 = (self.matrix[1][2] * rm11) + (self.matrix[2][2] * rm12);
        let tm13 = (self.matrix[1][3] * rm11) + (self.matrix[2][3] * rm12);

        self.matrix[2][0] = (self.matrix[1][0] * rm21) + (self.matrix[2][0] * rm22);
        self.matrix[2][1] = (self.matrix[1][1] * rm21) + (self.matrix[2][1] * rm22);
        self.matrix[2][2] = (self.matrix[1][2] * rm21) + (self.matrix[2][2] * rm22);
        self.matrix[2][3] = (self.matrix[1][3] * rm21) + (self.matrix[2][3] * rm22);

        self.matrix[1][0] = tm10;
        self.matrix[1][1] = tm11;
        self.matrix[1][2] = tm12;
        self.matrix[1][3] = tm13;

        self
    }

    pub fn rotate_y(&mut self, angle: f32) -> &mut Self {
        //self.mul(&Self::get_rotation_matrix_axis_y(angle));
        let cos: f32 = angle.cos();
        let sin: f32 = angle.sin();

        //rm - rotation matrix
        let rm00 = cos;
        let rm02 = -sin;
        let rm20 = sin;
        let rm22 = cos;

        //tm - temporary matrix
        let tm00 = (self.matrix[0][0] * rm00) + (self.matrix[2][0] * rm02);
        let tm01 = (self.matrix[0][1] * rm00) + (self.matrix[2][1] * rm02);
        let tm02 = (self.matrix[0][2] * rm00) + (self.matrix[2][2] * rm02);
        let tm03 = (self.matrix[0][3] * rm00) + (self.matrix[2][3] * rm02);

        self.matrix[2][0] = (self.matrix[0][0] * rm20) + (self.matrix[2][0] * rm22);
        self.matrix[2][1] = (self.matrix[0][1] * rm20) + (self.matrix[2][1] * rm22);
        self.matrix[2][2] = (self.matrix[0][2] * rm20) + (self.matrix[2][2] * rm22);
        self.matrix[2][3] = (self.matrix[0][3] * rm20) + (self.matrix[2][3] * rm22);

        self.matrix[0][0] = tm00;
        self.matrix[0][1] = tm01;
        self.matrix[0][2] = tm02;
        self.matrix[0][3] = tm03;
        self
    }

    pub fn rotate_z(&mut self, angle: f32) -> &mut Self {
        //self.mul(&Self::get_rotation_matrix_axis_z(angle));
        let cos: f32 = angle.cos();
        let sin: f32 = angle.sin();

        //rm - rotation matrix
        let rm00 = cos;
        let rm01 = sin;
        let rm10 = -sin;
        let rm11 = cos;

        //tm - temporary matrix
        let tm00 = (self.matrix[0][0] * rm00) + (self.matrix[1][0] * rm01);
        let tm01 = (self.matrix[0][1] * rm00) + (self.matrix[1][1] * rm01);
        let tm02 = (self.matrix[0][2] * rm00) + (self.matrix[1][2] * rm01);
        let tm03 = (self.matrix[0][3] * rm00) + (self.matrix[1][3] * rm01);

        self.matrix[1][0] = (self.matrix[0][0] * rm10) + (self.matrix[1][0] * rm11);
        self.matrix[1][1] = (self.matrix[0][1] * rm10) + (self.matrix[1][1] * rm11);
        self.matrix[1][2] = (self.matrix[0][2] * rm10) + (self.matrix[1][2] * rm11);
        self.matrix[1][3] = (self.matrix[0][3] * rm10) + (self.matrix[1][3] * rm11);

        self.matrix[0][0] = tm00;
        self.matrix[0][1] = tm01;
        self.matrix[0][2] = tm02;
        self.matrix[0][3] = tm03;
        self
    }

    pub fn rotate_xyz(&mut self, ax: f32, ay: f32, az: f32) -> &mut Self {
        //self.mul(&Self::get_rotation_matrix_xyz(ax, ay, az));
        let cos_x = ax.cos();
        let sin_x = ax.sin();
        let cos_y = ay.cos();
        let sin_y = ay.sin();
        let cos_z = az.cos();
        let sin_z = az.sin();

        let m_sin_x = -sin_x;
        let m_sin_y = -sin_y;
        let m_sin_z = -sin_z;

        //tm - temporary matrix

        //rotate x
        let tm10 = (self.matrix[1][0] * cos_x) + (self.matrix[2][0] * sin_x);
        let tm11 = (self.matrix[1][1] * cos_x) + (self.matrix[2][1] * sin_x);
        let tm12 = (self.matrix[1][2] * cos_x) + (self.matrix[2][2] * sin_x);
        let tm13 = (self.matrix[1][3] * cos_x) + (self.matrix[2][3] * sin_x);

        let tm20 = (self.matrix[1][0] * m_sin_x) + (self.matrix[2][0] * cos_x);
        let tm21 = (self.matrix[1][1] * m_sin_x) + (self.matrix[2][1] * cos_x);
        let tm22 = (self.matrix[1][2] * m_sin_x) + (self.matrix[2][2] * cos_x);
        let tm23 = (self.matrix[1][3] * m_sin_x) + (self.matrix[2][3] * cos_x);

        //rotate y
        let tm00 = (self.matrix[0][0] * cos_y) + (tm20 * m_sin_y);
        let tm01 = (self.matrix[0][1] * cos_y) + (tm21 * m_sin_y);
        let tm02 = (self.matrix[0][2] * cos_y) + (tm22 * m_sin_y);
        let tm03 = (self.matrix[0][3] * cos_y) + (tm23 * m_sin_y);
        self.matrix[2][0] = (self.matrix[0][0] * sin_y) + (tm20 * cos_y);
        self.matrix[2][1] = (self.matrix[0][1] * sin_y) + (tm21 * cos_y);
        self.matrix[2][2] = (self.matrix[0][2] * sin_y) + (tm22 * cos_y);
        self.matrix[2][3] = (self.matrix[0][3] * sin_y) + (tm23 * cos_y);

        //rotate z
        self.matrix[0][0] = (tm00 * cos_z) + (tm10 * sin_z);
        self.matrix[0][1] = (tm01 * cos_z) + (tm11 * sin_z);
        self.matrix[0][2] = (tm02 * cos_z) + (tm12 * sin_z);
        self.matrix[0][3] = (tm03 * cos_z) + (tm13 * sin_z);
        self.matrix[1][0] = (tm00 * m_sin_z) + (tm10 * cos_z);
        self.matrix[1][1] = (tm01 * m_sin_z) + (tm11 * cos_z);
        self.matrix[1][2] = (tm02 * m_sin_z) + (tm12 * cos_z);
        self.matrix[1][3] = (tm03 * m_sin_z) + (tm13 * cos_z);

        self
    }

    pub fn rotate(&mut self, rot: &Vec3f) -> &mut Self {
        self.rotate_x(rot[0]).rotate_y(rot[1]).rotate_z(rot[2]);
        //self.rotate_xyz(rot[0], rot[1], rot[2]);
        self

    }

    pub fn scale_xyz(&mut self, sx: f32, sy: f32, sz: f32) -> &mut Self {
        //self.mul(&Self::get_scaling_matrix_xyz(sx, sy, sz));
        self.matrix[0][0] = self.matrix[0][0] * sx;
        self.matrix[0][1] = self.matrix[0][1] * sx;
        self.matrix[0][2] = self.matrix[0][2] * sx;
        self.matrix[0][3] = self.matrix[0][3] * sx;

        self.matrix[1][0] = self.matrix[1][0] * sy;
        self.matrix[1][1] = self.matrix[1][1] * sy;
        self.matrix[1][2] = self.matrix[1][2] * sy;
        self.matrix[1][3] = self.matrix[1][3] * sy;

        self.matrix[2][0] = self.matrix[2][0] * sz;
        self.matrix[2][1] = self.matrix[2][1] * sz;
        self.matrix[2][2] = self.matrix[2][2] * sz;
        self.matrix[2][3] = self.matrix[2][3] * sz;

        self
    }

    pub fn scale(&mut self, sc: &Vec3f) -> &mut Self {
        self.scale_xyz(sc[0], sc[1], sc[2])
    }

    pub fn mul(&mut self, matrix: &Mat4f) -> &mut Self {
        let mut arg_mat = self.clone();


        for y in [0, 1, 2, 3] {
            for x in [0, 1, 2, 3] {
                for z in [0, 1, 2, 3] {
                    self.matrix[y][x] += (arg_mat.matrix[y][z] * matrix.matrix[z][x]);
                }
            }
        }
        /*
        self.matrix[0][0] = ((arg_mat.matrix[0][0] * matrix.matrix[0][0]) + (arg_mat.matrix[1][0] * matrix.matrix[0][1]) + (arg_mat.matrix[2][0] * matrix.matrix[0][2]) + (arg_mat.matrix[3][0] * matrix.matrix[0][3]));
        self.matrix[0][1] = ((arg_mat.matrix[0][1] * matrix.matrix[0][0]) + (arg_mat.matrix[1][1] * matrix.matrix[0][1]) + (arg_mat.matrix[2][1] * matrix.matrix[0][2]) + (arg_mat.matrix[3][1] * matrix.matrix[0][3]));
        self.matrix[0][2] = ((arg_mat.matrix[0][2] * matrix.matrix[0][0]) + (arg_mat.matrix[1][2] * matrix.matrix[0][1]) + (arg_mat.matrix[2][2] * matrix.matrix[0][2]) + (arg_mat.matrix[3][2] * matrix.matrix[0][3]));
        self.matrix[0][3] = ((arg_mat.matrix[0][3] * matrix.matrix[0][0]) + (arg_mat.matrix[1][3] * matrix.matrix[0][1]) + (arg_mat.matrix[2][3] * matrix.matrix[0][2]) + (arg_mat.matrix[3][3] * matrix.matrix[0][3]));

        self.matrix[1][0] = ((arg_mat.matrix[0][0] * matrix.matrix[1][0]) + (arg_mat.matrix[1][0] * matrix.matrix[1][1]) + (arg_mat.matrix[2][0] * matrix.matrix[1][2]) + (arg_mat.matrix[3][0] * matrix.matrix[1][3]));
        self.matrix[1][1] = ((arg_mat.matrix[0][1] * matrix.matrix[1][0]) + (arg_mat.matrix[1][1] * matrix.matrix[1][1]) + (arg_mat.matrix[2][1] * matrix.matrix[1][2]) + (arg_mat.matrix[3][1] * matrix.matrix[1][3]));
        self.matrix[1][2] = ((arg_mat.matrix[0][2] * matrix.matrix[1][0]) + (arg_mat.matrix[1][2] * matrix.matrix[1][1]) + (arg_mat.matrix[2][2] * matrix.matrix[1][2]) + (arg_mat.matrix[3][2] * matrix.matrix[1][3]));
        self.matrix[1][3] = ((arg_mat.matrix[0][3] * matrix.matrix[1][0]) + (arg_mat.matrix[1][3] * matrix.matrix[1][1]) + (arg_mat.matrix[2][3] * matrix.matrix[1][2]) + (arg_mat.matrix[3][3] * matrix.matrix[1][3]));

        self.matrix[2][0] = ((arg_mat.matrix[0][0] * matrix.matrix[2][0]) + (arg_mat.matrix[1][0] * matrix.matrix[2][1]) + (arg_mat.matrix[2][0] * matrix.matrix[2][2]) + (arg_mat.matrix[3][0] * matrix.matrix[2][3]));
        self.matrix[2][1] = ((arg_mat.matrix[0][1] * matrix.matrix[2][0]) + (arg_mat.matrix[1][1] * matrix.matrix[2][1]) + (arg_mat.matrix[2][1] * matrix.matrix[2][2]) + (arg_mat.matrix[3][1] * matrix.matrix[2][3]));
        self.matrix[2][2] = ((arg_mat.matrix[0][2] * matrix.matrix[2][0]) + (arg_mat.matrix[1][2] * matrix.matrix[2][1]) + (arg_mat.matrix[2][2] * matrix.matrix[2][2]) + (arg_mat.matrix[3][2] * matrix.matrix[2][3]));
        self.matrix[2][3] = ((arg_mat.matrix[0][3] * matrix.matrix[2][0]) + (arg_mat.matrix[1][3] * matrix.matrix[2][1]) + (arg_mat.matrix[2][3] * matrix.matrix[2][2]) + (arg_mat.matrix[3][3] * matrix.matrix[2][3]));

        self.matrix[3][0] = ((arg_mat.matrix[0][0] * matrix.matrix[3][0]) + (arg_mat.matrix[1][0] * matrix.matrix[3][1]) + (arg_mat.matrix[2][0] * matrix.matrix[3][2]) + (arg_mat.matrix[3][0] * matrix.matrix[3][3]));
        self.matrix[3][1] = ((arg_mat.matrix[0][1] * matrix.matrix[3][0]) + (arg_mat.matrix[1][1] * matrix.matrix[3][1]) + (arg_mat.matrix[2][1] * matrix.matrix[3][2]) + (arg_mat.matrix[3][1] * matrix.matrix[3][3]));
        self.matrix[3][2] = ((arg_mat.matrix[0][2] * matrix.matrix[3][0]) + (arg_mat.matrix[1][2] * matrix.matrix[3][1]) + (arg_mat.matrix[2][2] * matrix.matrix[3][2]) + (arg_mat.matrix[3][2] * matrix.matrix[3][3]));
        self.matrix[3][3] = ((arg_mat.matrix[0][3] * matrix.matrix[3][0]) + (arg_mat.matrix[1][3] * matrix.matrix[3][1]) + (arg_mat.matrix[2][3] * matrix.matrix[3][2]) + (arg_mat.matrix[3][3] * matrix.matrix[3][3]));
        */
        self
    }

    pub fn add(&mut self, matrix: &Mat4f) -> &mut Self {
        for x in [0, 1, 2, 3] {
            for y in [0, 1, 2, 3] {
                self.matrix[x][y] += matrix.matrix[x][y];
            }
        }
        self
    }

    pub fn sub(&mut self, matrix: &Mat4f) -> &mut Self {
        for x in [0, 1, 2, 3] {
            for y in [0, 1, 2, 3] {
                self.matrix[x][y] -= matrix.matrix[x][y];
            }
        }
        self
    }

    pub fn get_model_mat(pos: &Vec3f, rot: &Vec3f, sc: &Vec3f) -> Mat4f {
        let mut res = Mat4f::new();
        res.identity()
            .translate(pos)
            .rotate_xyz(-rot[0], -rot[1], -rot[2])
            .scale(sc);
        return res;
    }

    pub fn get_subchunk_model_mat(pos: &Vec3ub) -> Mat4f {
        let mut res = Mat4f::new();
        res.identity().translate(&[pos[0] as f32, pos[1] as f32, pos[2] as f32]);
        return res;
    }
}

/*

        a b c d
Mat4f = e f g h = [[a, e, i, m], [b, f, j, n]...]
        i j k l
        m n o p

Vec4f = {x; y; z; w}

Mat4f * Vec4f =
{x*a + y*b + z*c + w*d;
 x*e + y*f + z*g + w*h;
 x*i + y*j + z*k + w*l;
 x*m + y*n + z*o + w*p}

Vec4f * Mat4f =
{x*a + y*e + z*i + w*m;
 x*b + y*f + z*j + w*n;
 x*c + y*g + z*k + w*o;
 x*d + y*h + z*l + w*p}
 */

#[derive(Error, Debug)]
pub enum ShaderError {
    #[error("Error while compiling shader: {0}")]
    CompilationError(String),
    #[error("Error while linking shaders: {0}")]
    LinkingError(String),
    #[error{"{0}"}]
    Utf8Error(#[from] FromUtf8Error),
    #[error{"{0}"}]
    NulError(#[from] NulError),
    #[error{"{0}"}]
    ImageError(#[from] ImageError)
}