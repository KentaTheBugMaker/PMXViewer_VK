pub mod util {
    use std::fs::File;
    use std::io::BufReader;
    use std::path::Path;
    use std::sync::Arc;
    use std::thread;

    use PMXUtil::pmx_types::pmx_types::{PMXSphereMode, PMXTextureList, PMXVertex, PMXFace, PMXMaterial};
    use vk::buffer::{BufferUsage, CpuAccessibleBuffer};
    use vk::device::{Device, Queue};
    use vk::format::Format;
    use vk::image::{Dimensions, ImmutableImage};
    use vk::memory::pool::PotentialDedicatedAllocation;
    use vk::sync::GpuFuture;
    use vk::image::MipmapsCount;
    #[derive(Default, Copy, Clone)]
    pub struct Vertex {
        position: [f32; 4],
        normal: [f32; 4],
        uv: [f32; 2],
    }
    vulkano::impl_vertex!(Vertex,position,normal,uv);
    pub fn convert_to_vertex(vertex: &PMXVertex) -> Vertex {
        Vertex {
            position: [vertex.position[0], vertex.position[1], vertex.position[2], 1.0],
            normal: [vertex.norm[0], vertex.norm[1], vertex.norm[2], 0.0],
            uv: vertex.uv,
        }
    }

    pub fn convert_to_vertex_buffer(device: Arc<Device>, v: &[PMXVertex]) -> Arc<CpuAccessibleBuffer<[Vertex], PotentialDedicatedAllocation<vk::memory::pool::StdMemoryPoolAlloc>>> {
        let vertices = v.clone();
        let mut out = vec![];
        for elem in vertices {
            out.push(convert_to_vertex(elem));
        }
        CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage::vertex_buffer(), false, out.iter().cloned()).unwrap()
    }

    pub fn convert_to_index_buffer(device: Arc<Device>, triangles: &[PMXFace]) -> Arc<CpuAccessibleBuffer<[u32], PotentialDedicatedAllocation<vk::memory::pool::StdMemoryPoolAlloc>>> {
        let triangles = triangles;
        let mut indices = vec![];
        for elem in triangles {
            indices.push(elem.vertices[0]);
            indices.push(elem.vertices[1]);
            indices.push(elem.vertices[2]);
        }
        CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage::index_buffer(), false, indices.iter().cloned()).unwrap()
    }

    pub struct DrawAsset {
        pub    ibo: Arc<CpuAccessibleBuffer<[u32], PotentialDedicatedAllocation<vk::memory::pool::StdMemoryPoolAlloc>>>,
        pub    texture: usize,
        pub    toon_texture_index: usize,
        pub    diffuse: [f32; 4],
        pub    ambient: [f32; 3],
        pub    specular: [f32; 3],
        pub    specular_intensity: f32,
        pub    edge_flag: bool,
        pub    edge_color: [f32; 4],
        pub    sp: i32,
    }

    pub fn make_draw_asset(device: Arc<Device>, queue: Arc<Queue>, faces: Vec<PMXFace>, texture_list: &PMXTextureList, materials: &[PMXMaterial], filename: &str) -> (Vec<DrawAsset>, Vec<Arc<ImmutableImage<Format>>>) {
        let mut out = Vec::new();
        let mut v =  faces.clone();
        let mut end;
        let mut textures = Vec::new();
        let blank_texture_id = texture_list.textures.len();
        //Texture Load
        println!("Start Loading Textures...");
        let path = std::path::Path::new(&filename);
        let path = path.parent().unwrap().to_str().unwrap();
        for texture_name in &texture_list.textures {
            let path = path.to_string() + "/" + &texture_name.replace("\\", &std::path::MAIN_SEPARATOR.to_string());
            println!("path:{}", path);
            let path = Path::new(&path);
            let ext = path.extension().unwrap().to_str().unwrap();
            let read_as = match ext {
                "spa" => image::ImageFormat::Bmp,
                "sph" => image::ImageFormat::Bmp,
                _ => image::ImageFormat::from_path(&path).unwrap()
            };
            let reader = BufReader::new(File::open(path).unwrap());
            let qc = queue.clone();
            let join_handle = thread::spawn(move || {
                let image = image::load(reader, read_as).unwrap().to_rgba();
                let dimensions = image.dimensions();

                let (texture, texture_future) = ImmutableImage::from_iter(
                    image.iter().cloned(),
                    Dimensions::Dim2d { width: dimensions.0, height: dimensions.1 },
                    MipmapsCount::One ,Format::R8G8B8A8Srgb, qc.clone()).unwrap();
                let mut future = Some(Box::new(texture_future) as Box<dyn GpuFuture>);
                future.take().unwrap().flush().unwrap();
                texture
            });
            let texture = join_handle.join().unwrap();
            textures.push(texture);
        }
        let image = image::open("./blank.png").unwrap().to_rgba();

        let dimensions = image.dimensions();
        let (texture, texture_future) = ImmutableImage::from_iter(
            image.iter().cloned(),
            Dimensions::Dim2d { width: dimensions.0, height: dimensions.1 },
            MipmapsCount::One,
            Format::R8G8B8A8Srgb, queue.clone()).unwrap();
        let mut future = Some(Box::new(texture_future) as Box<dyn GpuFuture>);
        future.take().unwrap().flush().unwrap();

        textures.push(texture);
        println!("End Loading Textures...");
        for material in materials {
            end = (material.num_face_vertices / 3) as usize;//
            println!("len:{},({},{})", v.len(), 0, end);

            let v :Vec<PMXFace>= v.drain(0..end).collect();
            let faces = v;
            let ibo = convert_to_index_buffer(device.clone(), &faces);
            let sp = match material.spheremode {
                PMXSphereMode::None => { 0 }
                PMXSphereMode::Mul => { 1 }
                PMXSphereMode::Add => { 2 }
                PMXSphereMode::SubTexture => { 3 }
            };
            let mut toon_texture_index = material.toon_texture_index as usize;
            println!("Toon Texture Index={}", toon_texture_index);
            if toon_texture_index >= blank_texture_id {
                toon_texture_index = blank_texture_id;
            }
            let mut ti = material.texture_index as usize;//-1が渡されたときクラッシュ
            if ti >= blank_texture_id {
                ti = blank_texture_id;
            }
            let edge_flag = (material.drawmode & 0x10 > 1);
            let edge_color = material.edge_color;
            let asset = DrawAsset { ibo, texture: ti, toon_texture_index, diffuse: material.diffuse, ambient: material.ambient, specular: material.specular, specular_intensity: material.specular_factor, edge_flag, edge_color, sp };
            out.push(asset);
        }
        println!("End Create Render Asset");
        (out, textures)
    }
}