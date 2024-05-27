use cgmath::Vector2;



pub struct BoxCollider {
    //pos of the center of the collider
    pos: Vector2<f32>,
    size: Vector2<f32>,
}

impl BoxCollider {
    pub fn new(pos: Vector2<f32>, size: Vector2<f32>) -> Self {
        Self {
            pos,
            size,
        }
    }

    pub fn CheckCollision(&mut self, other: &BoxCollider) -> bool {
        let top_left = self.pos - self.size/2.0;
        let other_top_left = other.pos - other.size/2.0;

        let collisionX = top_left.x + self.size.x >= other_top_left.x && 
        other_top_left.x + other.size.x >= top_left.x;
        // collision y-axis?
        let collisionY = top_left.y + self.size.y >= other_top_left.y &&
            other_top_left.y + other.size.y >= top_left.y;
        // collision only if on both axes
        return collisionX && collisionY;
    }
}