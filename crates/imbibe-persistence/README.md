# imbibe-persistence

This crate details the database model of the information extracted from a block and its constituent transactions.

It uses [diesel](diesel.rs) and [postgresql](postgresql.org) to provide the read and write methods against the databse.

The sql files defining the schema for block and tx can be found in `migrations`.
