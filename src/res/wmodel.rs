use std::{collections::HashMap, error::Error};

use ash::vk::{self, PFN_vkCmdUpdateBuffer};
use gltf::{
  json::{
    extensions::{self, scene},
    texture,
  },
  texture::Sampler,
  Gltf, Image, Material, Semantic,
};
use nalgebra_glm::{vec2, vec4, Mat4, Vec2, Vec4};

use crate::{
  sys::{
    warenaitems::{WAIdxBuffer, WAIdxImage, WArenaItem},
    wcommandencoder::WCommandEncoder,
  },
  wvulkan::WVulkan,
};

pub struct WMesh {}

#[derive(Default)]
pub struct WNode {
  pub children: Vec<WNode>,
  pub mat: Mat4,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
// pub struct Vertex {
//     pub position: Vec4,
//     pub normal: Vec4,
//     pub color: Vec4,
//     pub uvs: Vec2,
// }

pub struct WVertex {
  pub position: Vec4,
  pub normal: Vec4,
  pub color: Vec4,
  pub uvs: Vec2,
}

pub struct WModel {
  //   pub vertBuff: Vec<WVertex>,
  //   pub top_node: WNode,
  //   pub indexBuff: Vec<u32>,
  //   pub textures: Vec<WAIdxImage>,
  pub vertices: Vec<WVertex>,
  pub indices: Vec<u32>,
  pub nodes: Vec<Node>,
  pub gpu_verts_buff: WAIdxBuffer,
  pub gpu_indices_buff: WAIdxBuffer,
  //   pub gpu_verts: WAIdxBuffer,
  //   pub gpu_indices: WAIdxBuffer,
  // pub images: Vec<gltf::Image<'a>>,
  // pub textures: Vec<gltf::Texture<'a>>,
  // pub samplers: Vec<Sampler<'a>>,
}

#[derive(Debug, Clone, Copy)]
pub struct Mesh {
  pub vertex_offset: u32,
  pub vertex_count: u32,
  pub index_offset: u32,
  pub index_count: u32,
  // pub material: gltf::Material,
}

#[derive(Debug, Clone, Copy)]
pub struct Node {
  pub transform: [[f32; 4]; 4],
  pub mesh: Mesh,
}

impl WModel {
  #[allow(unused_parens)]
  pub fn new(
    file_location: String,
    w_vulkan: &mut WVulkan,
  ) -> Self {
    // let mut vertBuff = vec![];
    // let mut indexBuff = vec![];
    // let mut textures = vec![];

    let root_models_dir = std::env::var("WORKSPACE_DIR").unwrap() + "\\src\\models\\";

    // let gltf = Gltf::open(root_models_dir + &file_location);

    let mut mesh_index_redirect = HashMap::<(usize, usize), usize>::new();

    let mut vertices = vec![];
    let mut indices = vec![];

    let mut meshes = vec![];
    let mut nodes = vec![];

    let (document, buffers, gltf_images) = gltf::import(
      // &path
      root_models_dir + &file_location,
    )
    .map_err(|e| {
      debug_assert!(false);
      // <dyn Error>::Load(e.to_string())
    })
    .unwrap();

    for mesh in document.meshes() {
      for primitive in mesh.primitives() {
        if (primitive.indices().is_some()
          && primitive.get(&gltf::Semantic::Positions).is_some()
          && primitive.get(&gltf::Semantic::Normals).is_some())
        {
          let og_index = (mesh.index(), primitive.index());

          if mesh_index_redirect.get(&og_index).is_none() {
            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

            // vertices
            let vertex_reader = reader.read_positions().unwrap();
            let vertex_offset = vertices.len() as _;
            let vertex_count = vertex_reader.len() as _;

            let normals = reader
              .read_normals()
              .unwrap()
              .map(|n| vec4(n[0], n[1], n[2], 0.0))
              .collect::<Vec<_>>();

            let colors = reader
              .read_colors(0)
              .map(|reader| reader.into_rgba_f32().map(Vec4::from).collect::<Vec<_>>());

            let uvs = reader
              .read_tex_coords(0)
              .map(|reader| reader.into_f32().map(Vec2::from).collect::<Vec<_>>());

            vertex_reader.enumerate().for_each(|(index, p)| {
              let position = vec4(p[0], p[1], p[2], 0.0);
              let normal = normals[index];
              let color = colors
                .as_ref()
                .map_or(vec4(1.0, 1.0, 1.0, 1.0), |colors| colors[index]);
              let uvs = uvs.as_ref().map_or(vec2(0.0, 0.0), |uvs| uvs[index]);

              vertices.push(WVertex {
                position,
                normal,
                color,
                uvs,
              });
            });

            // indices
            let index_reader = reader.read_indices().unwrap().into_u32();
            let index_offset = indices.len() as _;
            let index_count = index_reader.len() as _;

            index_reader.for_each(|i| indices.push(i));

            // material
            let material = primitive.material();

            let mesh_index = meshes.len();

            mesh_index_redirect.insert(og_index, mesh_index);

            meshes.push(Mesh {
              vertex_offset,
              vertex_count,
              index_offset,
              index_count,
              // material,
            });
          }
        }
      }
    }

    // fn is_primitive_supported(primitive: &extensions::mesh::Primitive) -> bool {
    //   primitive.indices().is_some()
    //     && primitive.get(&Semantic::Positions).is_some()
    //     && primitive.get(&Semantic::Normals).is_some()
    // }

    for node in document.nodes().filter(|n| n.mesh().is_some()) {
      let transform = node.transform().matrix();
      let gltf_mesh = node.mesh().unwrap();

      for primitive in gltf_mesh.primitives() {
        if (primitive.indices().is_some()
          && primitive.get(&gltf::Semantic::Positions).is_some()
          && primitive.get(&gltf::Semantic::Normals).is_some())
        {
          let og_index = (gltf_mesh.index(), primitive.index());
          let mesh_index = *mesh_index_redirect.get(&og_index).unwrap();
          let mesh = meshes[mesh_index];

          nodes.push(Node { transform, mesh })
        }
      }
    }

    // let images = gltf_images
    //   .iter()
    //   .map(Image::try_from)
    //   .collect::<Result<_>>()
    //   .unwrap();

    // Init samplers with a default one.
    // Textures with no sampler will reference this one.

    // let mut samplers = vec![gltf::texture::Sampler {
    // //   mag_filter: MagFilter::Linear,
    // //   min_filter: MinFilter::LinearMipmapLinear,
    // //   wrap_s: WrapMode::Repeat,
    // //   wrap_t: WrapMode::Repeat,
    // // ..Default
    //   document: &document.clone(),
    //   index: None,
    //   json: wmemzeroed!()
    // }];

    // document
    //   .samplers()
    //   .map(Sampler::from)
    //   .for_each(|s| samplers.push(s));

    // let textures = document.textures().map(Texture::from).collect::<Vec<_>>();

    // let textures = vec![];
    // let images = vec![];
    let mut vert_sz = (vertices.len());
    vert_sz = vert_sz * std::mem::size_of::<WVertex>();
    let mut gpu_verts_buff = w_vulkan.w_tl.new_buffer(
      &mut w_vulkan.w_device,
      vk::BufferUsageFlags::STORAGE_BUFFER,
      vert_sz as u32,
      false,
    ).0;
    unsafe{
        let _gpu_verts_buff = gpu_verts_buff.get_mut();

        _gpu_verts_buff.map(&w_vulkan.w_device.device);
        use std::ptr::copy_nonoverlapping as memcpy;
          memcpy(
            vertices.as_ptr(),
            _gpu_verts_buff.mapped_mems[0].cast(),
            vertices.len(),
          );
    }


    let mut vert_sz = (indices.len());
    vert_sz = vert_sz * std::mem::size_of::<u32>();
    let mut gpu_indices_buff = w_vulkan.w_tl.new_buffer(
      &mut w_vulkan.w_device,
      vk::BufferUsageFlags::STORAGE_BUFFER,
      vert_sz as u32,
      false,
    ).0;

    unsafe{
        let _gpu_indices_buff = gpu_indices_buff.get_mut();

        _gpu_indices_buff.map(&w_vulkan.w_device.device);
        use std::ptr::copy_nonoverlapping as memcpy;
          memcpy(
            indices.as_ptr(),
            _gpu_indices_buff.mapped_mems[0].cast(),
            indices.len(),
          );
    }


    // let command_encoder = WCommandEncoder::new();

    // let cmd_buff = w_vulkan.w_device.queue
    // let cmd_buff = w_vulkan.w_device.curr_pool().get_cmd_buff();

    unsafe {
    //   let cmd_buf_begin_info = vk::CommandBufferBeginInfo::builder();
    //   w_vulkan
    //     .w_device
    //     .device
    //     .begin_command_buffer(cmd_buff, &cmd_buf_begin_info)
    //     .unwrap();

    //   memcpy(
    //     indices.as_ptr(),
    //     gpu_indices_buff.1.mapped_mems[0].cast(),
    //     indices.len(),
    //   );

      // PFN_vkCmdUpdateBuffer();
      //   w_vulkan
      //     .w_device
      //     .device
      //     .cmd_update_buffer(cmd_buff, gpu_verts_buff, 0, vertices.as_ptr() as *mut u8);
    }

    // command_encoder.push(cmd_buff);

    WModel {
      vertices,
      indices,
      nodes,
      gpu_verts_buff: gpu_verts_buff,
      gpu_indices_buff: gpu_indices_buff,
      //   images,
      //   textures,
      //   samplers: wmemzeroed!(),
    }
  }
  fn create_gpu_buff() {}
}
// pub fn load_file<P: AsRef<Path>>(path: P) -> Result<Model> {
//     let (document, buffers, gltf_images) =
//         gltf::import(&path).map_err(|e| Error::Load(e.to_string()))?;

//     let mut vertices = vec![];
//     let mut indices = vec![];

//     let mut meshes = vec![];
//     let mut nodes = vec![];

//     let mut mesh_index_redirect = HashMap::<(usize, usize), usize>::new();

//     for mesh in document.meshes() {
//         for primitive in mesh.primitives().filter(is_primitive_supported) {
//             let og_index = (mesh.index(), primitive.index());

//             if mesh_index_redirect.get(&og_index).is_none() {
//                 let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

//                 // vertices
//                 let vertex_reader = reader.read_positions().unwrap();
//                 let vertex_offset = vertices.len() as _;
//                 let vertex_count = vertex_reader.len() as _;

//                 let normals = reader
//                     .read_normals()
//                     .unwrap()
//                     .map(|n| vec4(n[0], n[1], n[2], 0.0))
//                     .collect::<Vec<_>>();

//                 let colors = reader
//                     .read_colors(0)
//                     .map(|reader| reader.into_rgba_f32().map(Vec4::from).collect::<Vec<_>>());

//                 let uvs = reader
//                     .read_tex_coords(0)
//                     .map(|reader| reader.into_f32().map(Vec2::from).collect::<Vec<_>>());

//                 vertex_reader.enumerate().for_each(|(index, p)| {
//                     let position = vec4(p[0], p[1], p[2], 0.0);
//                     let normal = normals[index];
//                     let color = colors.as_ref().map_or(Vec4::ONE, |colors| colors[index]);
//                     let uvs = uvs.as_ref().map_or(Vec2::ZERO, |uvs| uvs[index]);

//                     vertices.push(Vertex {
//                         position,
//                         normal,
//                         color,
//                         uvs,
//                     });
//                 });

//                 // indices
//                 let index_reader = reader.read_indices().unwrap().into_u32();
//                 let index_offset = indices.len() as _;
//                 let index_count = index_reader.len() as _;

//                 index_reader.for_each(|i| indices.push(i));

//                 // material
//                 let material = primitive.material().into();

//                 let mesh_index = meshes.len();

//                 mesh_index_redirect.insert(og_index, mesh_index);

//                 meshes.push(Mesh {
//                     vertex_offset,
//                     vertex_count,
//                     index_offset,
//                     index_count,
//                     // material,
//                 });
//             }
//         }
//     }

//     for node in document.nodes().filter(|n| n.mesh().is_some()) {
//         let transform = node.transform().matrix();
//         let gltf_mesh = node.mesh().unwrap();

//         for primitive in gltf_mesh.primitives().filter(is_primitive_supported) {
//             let og_index = (gltf_mesh.index(), primitive.index());
//             let mesh_index = *mesh_index_redirect.get(&og_index).unwrap();
//             let mesh = meshes[mesh_index];

//             nodes.push(Node { transform, mesh })
//         }
//     }

//     let images = gltf_images
//         .iter()
//         .map(Image::try_from)
//         .collect::<Result<_>>()?;

//     // Init samplers with a default one.
//     // Textures with no sampler will reference this one.
//     let mut samplers = vec![Sampler {
//         mag_filter: MagFilter::Linear,
//         min_filter: MinFilter::LinearMipmapLinear,
//         wrap_s: WrapMode::Repeat,
//         wrap_t: WrapMode::Repeat,
//     }];
//     document
//         .samplers()
//         .map(Sampler::from)
//         .for_each(|s| samplers.push(s));

//     let textures = document.textures().map(Texture::from).collect::<Vec<_>>();

//     Ok(Model {
//         vertices,
//         indices,
//         nodes,
//         images,
//         textures,
//         samplers,
//     })
// }

// pub struct WModel{
//     pub vertBuff: Vec<WVertex>,
//     pub top_node: WNode,
//     pub indexBuff: Vec<u32>,
//     pub textures: Vec<WAIdxImage>,
// }

// impl WModel{
//     pub fn new(
//         file_location: String,
//     )->Self{
//         let mut vertBuff = vec![];
//         let mut indexBuff = vec![];
//         let mut textures = vec![];

//         let root_models_dir = std::env::var("WORKSPACE_DIR").unwrap() + "\\src\\models\\";

//         let gltf = Gltf::open(
//             root_models_dir + &file_location
//         );

//         macro_rules! load_node {
//             ($node: expr) => {

//             };
//         }

//         fn load_node(
//             gltf_node: &gltf::Node,
//             w_parent_node: &mut WNode,
//             vertBuff: &mut Vec<WVertex>,
//             indexBuff: &mut Vec<u32>,
//             ){
//             println!(
//                 "Node #{} has {} children",
//                 gltf_node.index(),
//                 gltf_node.children().count(),
//             );
//             let mut curr_node = WNode{..wdef!()};

//             curr_node.mat = Mat4::identity();
//             // if(gltf_node.transform().)

//             let tranf_mat = gltf_node.transform().matrix();

//             unsafe{
//                 curr_node.mat[0] = tranf_mat[0][0];
//                 curr_node.mat[1] = tranf_mat[0][1];
//                 curr_node.mat[2] = tranf_mat[0][2];
//                 curr_node.mat[3] = tranf_mat[0][3];
//                 curr_node.mat[4] = tranf_mat[1][0];
//                 curr_node.mat[5] = tranf_mat[1][1];
//                 curr_node.mat[6] = tranf_mat[1][2];
//                 curr_node.mat[7] = tranf_mat[1][3];
//                 curr_node.mat[8] = tranf_mat[2][0];
//                 curr_node.mat[9] = tranf_mat[2][1];
//                 curr_node.mat[10] = tranf_mat[2][2];
//                 curr_node.mat[11] = tranf_mat[2][3];
//                 curr_node.mat[12] = tranf_mat[3][0];
//                 curr_node.mat[13] = tranf_mat[3][1];
//                 curr_node.mat[14] = tranf_mat[3][2];
//                 curr_node.mat[15] = tranf_mat[3][3];
//             }

//             if let Some(mesh) = gltf_node.mesh(){
//                 for primitive in mesh.primitives(){
//                     if(
//                         primitive.indices().is_some()
//                         && primitive.get(&gltf::Semantic::Positions).is_some()
//                         && primitive.get(&gltf::Semantic::Normals).is_some()
//                     ){
//                         // let prim_type = mesh.pr
//                         let mesh_idx = mesh.index();
//                         let prim_idx = primitive.index();

//                         let mesh_idx = ;

//                         let first_id = primi

//                     }
//                     // primitive.
//                 }

//             }

//             for node in gltf_node.children(){
//                 load_node(&node,&mut curr_node, vertBuff, indexBuff);
//             }

//             w_parent_node.children.push(curr_node);

//             // fo

//         }

//         let mut top_node = WNode{..wdef!()};
//         match gltf {
//             Ok(__) => {
//                 // __.
//                 for scene in __.scenes(){

//                     for node in scene.nodes(){
//                         load_node(&node,&mut top_node, &mut vertBuff, &mut indexBuff);
//                     }
//                 }
//             },
//             Err(__) => {
//                 debug_assert!(false);
//             },
//         }
//         WModel { vertBuff, top_node, indexBuff, textures }

//     }

// }