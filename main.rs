

struct TopLevel {
    value : RefCell<BottomLevel>,
}

impl<'a> TopLevel {
    fn read(&'a self, buff : &'a mut [u8] ) -> MidLevel<'a> {
        MidLevel::new(buff, &self.value)
    }
}

struct MidLevel<'a> {
    buff : &'a mut [u8],
    reference : &'a RefCell<BottomLevel>,
}

impl<'a> MidLevel<'a> {
    fn new(buff : &'a mut [u8], stream : &'a RefCell<BottomLevel>) -> Self {
        Self {
            buff,
            reference,
        }
    }
}

impl MidLevel<'_> {
    fn func(mut self: Pin<&mut Self>) {
        let mut stream_borrow = self.reference.borrow_mut();
        let _ = stream_borrow.read(self.buff);   
    }
}



fn start<'a>(r : &'a i32, s : &'a i32) {
    
    let mut vecto = Vec::new();


    
    vecto.push(Rc::new(Referential { lref: r }));
    vecto.push(Rc::new(Referential { lref: s }));

    
    for i in vecto.iter() {
        println!("{}", i.lref);
    }
       
    
}

fn main() {

    let val1 : i32 = 0;
    let val2 : i32 = 5;
    start( &val1, &val2 );
    

    println!("Hello, world!");
}
