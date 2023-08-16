create table stored_event(
   id serial primary key not null,
   name text not null,
   payload text not null
);

create table author(
   id serial primary key not null,
   first_name text not null,
   last_name text not null,
   full_name text not null
);

create table book(
   id serial primary key not null,
   name text not null,
   pages_count int not null
);

create table author_book(
   author_id serial not null,
   book_id serial not null,
   constraint fk_author foreign key(author_id) references author(id),
   constraint fk_book foreign key(book_id) references book(id),
   primary key(author_id, book_id)
);