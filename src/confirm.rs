pub struct Confirm {
    action: Box<dyn Fn() -> ()>,
    confirm_header: String 
}

impl Confirm{
    pub fn new(Header: String, action: Box<dyn Fn()>) ->{
        return Ok(Self{
            confirm_header: Header,
            action,

        })
    }

}

