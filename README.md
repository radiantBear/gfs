# Grandfather-Father-Son Backup Utility

This program accepts contents that need to be backed up from `STDIN` and produces backup
files in the CWD. It is intended to enable easy backups from tools like `mariadb-dump`
that provide a current snapshot on `STDOUT`.

Backup files are maintained using the grandfather-father-son backup scheme to preserve
long-term backups while minimizing bloat. In particular, it keeps all backups from the
last week, one backup per week for the preceding month, and one backup per month in
perpetuity. Each time this program is run, **it will delete** all backups that aren't
needed to maintain this scheme.

## Installation

Right now, you must clone this repository and build the project (i.e. using `cargo`).

## Usage

To use this utility, simply pipe the contents you would like to back up into it,
optionally specifying a filename suffix to expect on files. The utility expects filenames
to match the YYYY-MM-DD format (plus the optional suffix). For example, this will create a
file like `2026-06-23.sql` and prune unneeded files from the current directory:

```console
mariadb@localhost:~/backups$ mariadb-dump database -uroot -p$MYSQL_PWD | gfs --suffix ".sql"
```
