
pub struct Rectangle{
    pos:[f32;2],
    size:[f32;2],
}

impl Rectangle {
    /// create new rectangle
    pub fn new(x:f32,y:f32,width:f32,height:f32)->Self{
        Self{
            pos: [x,y],
            size: [width,height]
        }
    }
    /// exist
    /**
    // ________
    // |__| | |
    // |_*__| |
    // |______|
    // **/
    pub fn exist(&self,pos:[f32;2])->bool{
        let x2=self.pos[0]+self.size[0];
        let y2=self.pos[1]+self.size[1];
        self.pos[0]<pos[0]&& pos[0]<x2 &&self.pos[1]<pos[1] && pos[1]< y2
    }
    pub fn area_size(&self)->f32{
        self.size[0]*self.size[1]
    }

}