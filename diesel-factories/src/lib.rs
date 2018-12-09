//! ```
//! #[macro_use]
//! extern crate diesel;
//!
//! use diesel::prelude::*;
//! use diesel::pg::PgConnection;
//! use diesel_factories::{Factory, InsertFactory, DefaultFactory};
//!
//! table! {
//!     users (id) {
//!         id -> Integer,
//!         name -> Text,
//!         age -> Integer,
//!     }
//! }
//!
//! #[derive(Queryable, Identifiable)]
//! pub struct User {
//!     pub id: i32,
//!     pub name: String,
//!     pub age: i32,
//! }
//!
//! #[derive(Insertable, Factory)]
//! #[table_name = "users"]
//! #[factory_model(User)]
//! pub struct UserFactory {
//!     name: String,
//!     age: i32,
//! }
//!
//! impl Default for UserFactory {
//!     fn default() -> UserFactory {
//!         UserFactory {
//!             name: "Bob".into(),
//!             age: 30,
//!         }
//!     }
//! }
//!
//! fn main() {
//!     use self::users::dsl::*;
//!
//!     let database_url = "postgres://localhost/diesel_factories_test";
//!     let con = PgConnection::establish(&database_url).unwrap();
//!     # con.begin_test_transaction();
//!
//!     // Create a new using our factory, overriding the default name
//!     let user = User::factory().name("Alice").insert::<User, _, _>(&con);
//!     assert_eq!("Alice", user.name);
//!     assert_eq!(30, user.age);
//!
//!     // Verifing that the user is in fact in the database
//!     let user_from_db = users
//!             .filter(id.eq(user.id))
//!             .first::<User>(&con)
//!             .unwrap();
//!     assert_eq!("Alice", user_from_db.name);
//!     assert_eq!(30, user_from_db.age);
//! }
//! ```

use diesel::associations::HasTable;
use diesel::backend::{Backend, SupportsReturningClause};
use diesel::connection::Connection;
use diesel::insertable::CanInsertInSingleQuery;
use diesel::prelude::*;
use diesel::query_builder::QueryFragment;
use diesel::sql_types::HasSqlType;
use std::default::Default;

pub use diesel_factories_code_gen::Factory;

pub trait DefaultFactory<T: Default> {
    fn factory() -> T {
        T::default()
    }
}

pub trait InsertFactory {
    fn insert<Model, Con, DB>(self, con: &Con) -> Model
    where
        Self: Insertable<<Model as HasTable>::Table>,
        <Self as Insertable<<Model as HasTable>::Table>>::Values:
            CanInsertInSingleQuery<DB> + QueryFragment<DB>,
        Con: Connection<Backend = DB>,
        DB: 'static
            + Backend
            + SupportsReturningClause
            + HasSqlType<<<<Model as HasTable>::Table as Table>::AllColumns as Expression>::SqlType>,
        Model: HasTable
            + Queryable<
                <<<Model as HasTable>::Table as Table>::AllColumns as Expression>::SqlType,
                DB,
            >,
        <<Model as HasTable>::Table as Table>::AllColumns: QueryFragment<DB>,
        <<Model as HasTable>::Table as QuerySource>::FromClause: QueryFragment<DB>;
}

impl<Factory> InsertFactory for Factory {
    fn insert<Model, Con, DB>(self, con: &Con) -> Model
    where
        Self: Insertable<<Model as HasTable>::Table>,
        <Self as Insertable<<Model as HasTable>::Table>>::Values:
            CanInsertInSingleQuery<DB> + QueryFragment<DB>,
        Con: Connection<Backend = DB>,
        DB: 'static
            + Backend
            + SupportsReturningClause
            + HasSqlType<<<<Model as HasTable>::Table as Table>::AllColumns as Expression>::SqlType>,
        Model: HasTable
            + Queryable<
                <<<Model as HasTable>::Table as Table>::AllColumns as Expression>::SqlType,
                DB,
            >,
        <<Model as HasTable>::Table as Table>::AllColumns: QueryFragment<DB>,
        <<Model as HasTable>::Table as QuerySource>::FromClause: QueryFragment<DB>,
    {
        let res = diesel::insert_into(Model::table())
            .values(self)
            .get_result::<Model>(con);

        match res {
            Ok(record) => record,
            Err(err) => panic!("{}", err),
        }
    }
}