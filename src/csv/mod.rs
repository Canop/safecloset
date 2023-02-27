use {
    crate::core::Entry,
    char_reader::CharReader,
    std::{
        fs::File,
        io,
        path::Path,
    },
};

#[derive(Debug, Default)]
pub struct Csv {
    pub rows: Vec<Vec<String>>,
}

impl Csv {
    /// create a new CSV from the content of the given reader, which is
    /// consumed until EOF.
    /// The separator is the cell separator, usually the comma (',')
    pub fn new<R: io::Read>(
        src: R,
        separator: char,
    ) -> io::Result<Self> {
        let mut csv = Self::default();
        csv.parse(src, separator)?;
        Ok(csv)
    }

    pub fn from_path(
        path: &Path,
        separator: char,
    ) -> io::Result<Self> {
        let file = File::open(path)?;
        Self::new(file, separator)
    }

    /// Consume the reader till EOF, adding all the
    /// rows found
    pub fn parse<R: io::Read>(
        &mut self,
        src: R,
        separator: char,
    ) -> io::Result<()> {
        let mut reader = CharReader::new(src);
        let mut row = Vec::new();
        let mut cell = String::new();
        let mut quoted = false;
        while let Some(c) = reader.next_char()? {
            if quoted {
                if c == '"' {
                    if let Ok(Some('"')) = reader.peek_char() {
                        // a sequence of 2 '"' is an escaped quote
                        cell.push('"');
                        _ = reader.next_char(); // just consuming the quote
                    } else {
                        quoted = false;
                        // next char should end the cell or row
                        // (or it's an illformed cell)
                    }
                } else {
                    cell.push(c);
                }
            } else {
                match c {
                    '"' => {
                        // the csv format mandates that it's the first
                        // char of the cell, we don't check that
                        quoted = true;
                    }
                    c if c == separator => {
                        let mut new_cell = String::new();
                        std::mem::swap(&mut cell, &mut new_cell);
                        row.push(new_cell);
                        quoted = false;
                    }
                    '\r' => {
                        // it's either invalid or part of a CRLF,
                        // we ignore it in both cases
                    }
                    '\n' => {
                        let mut new_cell = String::new();
                        std::mem::swap(&mut cell, &mut new_cell);
                        row.push(new_cell);
                        let mut new_row = Vec::new();
                        std::mem::swap(&mut row, &mut new_row);
                        self.rows.push(new_row);
                        quoted = false;
                    }
                    _ => {
                        cell.push(c);
                    }
                }
            }
        }
        let mut new_cell = String::new();
        std::mem::swap(&mut cell, &mut new_cell);
        row.push(new_cell);
        let mut new_row = Vec::new();
        std::mem::swap(&mut row, &mut new_row);
        self.rows.push(new_row);
        Ok(())
    }
    #[allow(dead_code)]
    pub fn row_count(&self) -> usize {
        self.rows.len()
    }
    pub fn col_count(&self) -> usize {
        self.rows.iter().map(|row| row.len()).max().unwrap_or(0)
    }
    pub fn into_entries(mut self) -> io::Result<Vec<Entry>> {
        let cols = self.col_count();
        if cols < 2 {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Not enough columns ({cols})"),
            ));
        }
        let entries = self
            .rows
            .drain(..)
            .filter_map(|mut row| {
                if row.len() >= 2 {
                    let name = row.remove(0);
                    let value = row.swap_remove(0);
                    Some(Entry::new(name, value))
                } else {
                    None
                }
            })
            .collect();
        Ok(entries)
    }
}

#[test]
fn test_read_csv() {
    let con = "A1,B1\nA2,\"B,2\",\"\"\"\",D2\nA3 ";
    let csv = Csv::new(con.as_bytes(), ',').unwrap();
    dbg!(&csv);
    assert_eq!(csv.rows.len(), 3);
    let row = &csv.rows[0];
    assert_eq!(row.len(), 2);
    assert_eq!(row[0], "A1");
    assert_eq!(row[1], "B1");
    let row = &csv.rows[1];
    assert_eq!(row.len(), 4);
    assert_eq!(row[0], "A2");
    assert_eq!(row[1], "B,2");
    assert_eq!(row[2], r#"""#);
    assert_eq!(row[3], "D2");
    let row = &csv.rows[2];
    assert_eq!(row.len(), 1);
    assert_eq!(row[0], "A3 ");
}
