#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;
#[macro_use] extern crate rocket_contrib;

use rocket_contrib::json::{Json, JsonValue};
use serde::{Deserialize, Serialize};
use rocket::response::content::Html;

#[derive(Debug, Serialize, Deserialize, Clone)] // add Clone trait here
struct Book {
    id: u32,
    title: String,
    author: String,
    description: String,
}


// Define a static variable to hold all the books
static mut BOOKS: Vec<Book> = Vec::new();

#[get("/books")]
fn get_all_books() -> Json<Vec<Book>> {
    // Use an unsafe block to access the global variable
    unsafe {
        Json(BOOKS.clone())
    }
}

#[get("/books/<id>")]
fn get_book(id: u32) -> Option<Json<Book>> {
    // Use an unsafe block to access the global variable
    unsafe {
        BOOKS.iter()
        .find(|book| book.id == id)
        .map(|book| Json(book.clone()))
    }
}

#[post("/books", format = "json", data = "<book>")]
fn create_book(book: Json<Book>) -> JsonValue {
    let new_book = book.into_inner();

    // Use an unsafe block to access the global variable
    unsafe {
        BOOKS.push(new_book.clone());
    }

    json!({
        "status": "success",
        "message": "Book created successfully",
        "data": new_book
    })
}

#[get("/")]
fn index() -> Html<&'static str> {
    // let html_str = fs::read_to_string("book.html").unwrap().clone();
    let html_str = include_str!("book.html");
    Html(&html_str)
}

fn main() {
    // Initialize the global variable with some books
    unsafe {
        BOOKS = vec![
            Book {
                id: 1,
                title: "The Lord of the Rings".to_string(),
                author: "J.R.R. Tolkien".to_string(),
                description: "The Lord of the Rings is an epic high fantasy novel by the English author and scholar J. R. R. Tolkien.".to_string(),
            },
            Book {
                id: 2,
                title: "The Hobbit".to_string(),
                author: "J.R.R. Tolkien".to_string(),
                description: "The Hobbit, or There and Back Again is a children's fantasy novel by English author J. R. R. Tolkien.".to_string(),
            }
        ];
    }

    rocket::ignite()
        .mount("/", routes![get_all_books, get_book, create_book, index])
        .launch();
}
