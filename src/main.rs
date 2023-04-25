#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;
#[macro_use] extern crate rocket_contrib;


use rocket_contrib::json::{Json, JsonValue};
use serde::{Deserialize, Serialize};
use rocket::response::content::Html;
use mongodb::{Client, Collection};
use futures::{stream::TryStreamExt, TryFutureExt};
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

#[post("/books", format = "json", data = "<book>")]
fn create_book(book: Json<Book>) -> Result<Status, Status> {
    let books_collection = Runtime::new().unwrap().block_on(establish_connection());

    println!("{}", books_collection.name());
    
    let new_book = Book {
        id: book.id,
        author: book.author.to_string(),
        description: book.description.to_string(),
        title: book.title.to_string()
    };

    println!("{:?}",new_book);
    
    
    let insertResult = Runtime::new().unwrap().block_on(books_collection.insert_one(new_book, None));

    match insertResult {
        Ok(insertResult) => Ok(Status::new(200,"Success")),
        Err(err) => {
            println!("{}",err);
            Err(Status::new(503, "Insert error"))
        }
    }
    // Ok(Status::new(200,"Success"))

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
