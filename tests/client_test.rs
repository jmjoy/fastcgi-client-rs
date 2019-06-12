use fastcgi_client::{ClientBuilder, Params, Address};
use std::error::Error;
use std::io;

#[test]
fn test_client() {
    env_logger::init();

    let response = ClientBuilder::new(Address::Tcp("127.0.0.1", 9000))
        .build()
        .unwrap()
        .do_request(Params::new(), &mut io::empty());

//    let response = ClientBuilder::new(Address::Tcp("127.0.0.1", 9000))
//        .build()
//        .unwrap()
//        .do_request(
//            Params::with(
//                "GET",
//                "/home/jmjoy/workspace/rust/fastcgi-client-rs/tests/php/index.php",
//                "",
//                "/index.php",
//                "/index.php",
//                "127.0.0.1",
//                "12345",
//                "127.0.0.1",
//                "80",
//                "jmjoy-PC",
//                "",
//                "0",
//            ),
//            &mut io::empty(),
//        );

    //    let _: () = response;
//    dbg!(&response.map(|buf| String::from_utf8(buf ).unwrap()));

    //    if let Err(ref e) = response {
    //        //        let e: &Fail = e;
    //        dbg!(e.backtrace());
    //
    //        //        if let Some(bt) = e.as_fail().and_then(|cause| cause.backtrace()) {
    //        //            println!("{}", bt)
    //        //        }
    //    }
    //
    //    //    response.map_err(|e| {
    //    ////        dbg!(e.source());
    //    //    }).map(|x| {
    //    //        dbg!(x);
    //    //    });
}


