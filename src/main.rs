#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;
#[macro_use] extern crate rocket_contrib;


use rocket_contrib::json::{Json};
use serde::{Deserialize, Serialize};
use rocket::response::content::Html;
use mongodb::{bson::doc, Client, options::{ClientOptions, ServerApi, ServerApiVersion}, Collection};
use futures::{stream::TryStreamExt};
use tokio::runtime::Runtime;
use std::{error::Error};
use rocket::http::Status;

#[derive(Debug, Serialize, Deserialize, Clone)] // add Clone trait here
struct Book {
    id: u32,
    title: String,
    author: String,
    description: String,
}

static USER_PWD: &str = "anhle:K!llboss300";

// Define a static variable to hold all the books
static mut BOOKS: Vec<Book> = Vec::new();

async fn establish_connection() -> Collection<Book> {
    // Connect to MongoDB
    let uri = "mongodb+srv://".to_owned() + USER_PWD + "@rust-server-db.edsxatn.mongodb.net/?retryWrites=true&w=majority";
    let client = Client::with_uri_str(uri).await.unwrap();
    let db = client.database("books_db");
    // Get a handle to the "books" collection in the "books_db" database
    db.collection::<Book>("books")
}

async fn write_to_database(new_book: Book) -> mongodb::error::Result<()>  {
    // Connect to MongoDB
    let uri = "mongodb+srv://".to_owned() + USER_PWD + "@rust-server-db.edsxatn.mongodb.net/?retryWrites=true&w=majority";

    let mut client_options = ClientOptions::parse(uri).await?;
    let server_api = ServerApi::builder().version(ServerApiVersion::V1).build();

    client_options.server_api = Some(server_api);

    let client = Client::with_options(client_options)?;

    client
        .database("admin")
        .run_command(doc! {"ping": 1}, None)
        .await?;

    let db = client.database("books_db");

    let book_collection = db.collection::<Book>("books");

    book_collection.insert_one(new_book, None).await?;

    println!("Pinged your deployment, you successfully connected to MongoDB!!");

    Ok(())
}

async fn get_book_from_database() -> Result<Vec<Book>, Box<dyn Error>> {
    let books_collection = establish_connection().await;
    let cursor = books_collection.find(None, None).await?;
    let books = cursor.try_collect().await;
    match books {
        Ok(books) => Ok(books),
        Err(err) => Err(format!("Cannot find all books in the database {}", err).into())
    }

}

#[get("/books")]
fn get_all_books() -> Result<Json<Vec<Book>>, Status> {
    // Use an unsafe block to access the global variable

    let books = Runtime::new().unwrap().block_on(get_book_from_database());

    match books {
        Ok(books) => Ok(Json(books)),
        Err(_) => Err(Status::new(404, "No books found in database")) 
    }
}

#[get("/books/<id>")]
fn get_book(id: u32) -> Result<Json<Book>, Status> {
    // Use an unsafe block to access the global variable
    let books = Runtime::new().unwrap().block_on(get_book_from_database());
    match books {
        Ok(books) => {
            let book = books
                                            .iter()
                                            .find(|book| book.id == id)
                                            .map(|book| Json(book.clone()));
            if !book.is_none()
            {
                Ok(book.unwrap())
            }
            else {
                Err(Status::new(404, "Id not found"))        
            }
        }
        Err(_) => Err(Status::new(404, "No books found in database"))
    }
    


}

#[post("/books", data = "<book_str>")]
fn create_book(book_str: String) -> Result<Status, Status> {

    let book: Vec<&str> = book_str.split("&").collect();
    let id: Vec<&str>  = book[0].split("=").collect();
    let title: Vec<&str>  = book[1].split("=").collect();
    let author: Vec<&str>  = book[2].split("=").collect();
    let description: Vec<&str>  = book[3].split("=").collect();

    let new_book = Book {
        id: id[1].parse::<u32>().unwrap(),
        title: title[1].replace("+", " ").to_string(),
        author: author[1].replace("+", " ").to_string(),
        description: description[1].replace("+", " ").to_string()
    };

    println!("{:?}",new_book);

    let _ = Runtime::new().unwrap().block_on(write_to_database(new_book));
        
    Ok(Status::new(200,"Success"))
}



#[get("/")]
fn index() -> Html<&'static str> {
    // let html_str = fs::read_to_string("book.html").unwrap().clone();
    let html_str = include_str!("book.html");
    Html(&html_str)
}

// #[catch(404)]
// fn not_found() -> Html<&'static str> {
//     let html_str = include_str!("book.html");
//     Html(&html_str)
// }

fn main() {
    rocket::ignite()
        // .register(catchers![not_found])
        .mount("/", routes![get_all_books, get_book, create_book, index])
        .launch();
}
