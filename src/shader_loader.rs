pub mod shader_loader_optifine {
    use std::fs::File;
    use std::io::Read;
    use std::path::Path;

    use shaderc::Compiler;

    use self::shaderc::{CompilationArtifact, ShaderKind};

    extern crate shaderc;

    pub struct Loader {
        shaders: Vec<CompileParameters>
    }

    struct CompileParameters {
        target: ShaderKind,
        source_text: String,
        source_name: String,

    }

    impl CompileParameters {
        fn open<P: AsRef<Path>>(path: P) -> Self {
            let ext = path.as_ref().extension().unwrap().to_str().unwrap();
            let target = match ext {
                "vsh" => ShaderKind::Vertex,
                "gsh" => ShaderKind::Geometry,
                "fsh" => ShaderKind::Fragment,
                _ => panic!("Error invalid shader extension")
            };

            let mut source_text = String::new();
            File::open(&path).unwrap().read_to_string(&mut source_text);
            //Transpile
            //End Transpile
            let source_name = path.as_ref().to_str().unwrap().to_string();
            Self {
                target,
                source_text,
                source_name,
            }
        }
    }

    impl Loader {
        fn open<P: AsRef<Path>>(path: P) -> Self {
            let mut shaders = Vec::new();
            Loader {
                shaders
            }
        }
        fn compile(&self) -> CompilationArtifact {
            let mut compiler = Compiler::new().unwrap();
            let source = "";
            let source_name = "vert.glsl";
            let target = ShaderKind::Vertex;
            let artifact = compiler.compile_into_spirv(source, ShaderKind::Vertex, "vert.glsl", "main", None).unwrap();
            artifact.as_binary_u8();
            artifact
        }
        //Unzip -> Transpile -> Compile(Shaderc)->Load binary
        //Refer  Runtime shader Differed Rendering
    }
}