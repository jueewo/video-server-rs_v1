# Project structure

```
video-server-rs/
├── Cargo.toml
├── src/
│   ├── main.rs
│   └── schema.sql  (initial DB schema)
├── storage/
│   ├── public/
│   │   └── welcome/
│   │       ├── master.m3u8
│   │       └── ...segments.ts
│   └── private/
│       └── lesson1/
│           ├── master.m3u8
│           └── ...segments.ts
└── migrations/  (SQLx will create this)
```


## Inject data

```
sqlite3 video.db < schema.sql
```
