Zero 2 Production
---

Following along with the book [Zero to Production in Rust](https://www.zero2prod.com/), with some modifications of my own design.

# Project Structure

Directory list:
 - docs/
 	- Markdown based documentation and notes (mostly empty for now).
 - migrations/
 	- PostgreSQL migration files.
 	- New migrations are generated through the SQLx command line tool: `sqlx migrate add <migration_description>`
 - scripts/
 	- Bash scripts for ease of development.
 - server/
 	- Source code for the HTTP server of the newsletter service, and integration tests
 - settings/
 	- YAML-based settings for different runtime environments.
 - zero2prod/
 	- Source code for business logic and domain-specific objects.
 	- client/
 		- REST clients for external services used by the application
 	- crypto/
 		- Cryptography-based code (e.g. user-identifying tokens)
 	- domain/
 		- Domain objects for parsing and consistency
 	- repo/
 		- Methods for interfacing with stored data
