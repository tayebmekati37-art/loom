use super::ast::*;

pub fn parse_picture(input: &str) -> Result<PictureClause, String> {

    let mut text = input.trim().to_uppercase();

    if text.starts_with("PIC ") {
        text = text[4..].trim().to_string();
    }

    let mut signed = false;

    if text.starts_with('S') {
        signed = true;
        text = text[1..].to_string();
    }

    let usage = Usage::Display;

    //--------------------------------------------------
    // PIC X
    //--------------------------------------------------

    if text == "X" {

        return Ok(PictureClause{
            signed,
            category: PictureCategory::Alphanumeric,
            length:1,
            scale:0,
            usage,
        });

    }

    //--------------------------------------------------
    // PIC X(n)
    //--------------------------------------------------

    if text.starts_with("X(") {

        if let Some(end)=text.find(')'){

            let len=&text[2..end];

            if let Ok(length)=len.parse::<usize>(){

                return Ok(PictureClause{
                    signed,
                    category:PictureCategory::Alphanumeric,
                    length,
                    scale:0,
                    usage,
                });

            }
        }
    }

    //--------------------------------------------------
    // PIC A
    //--------------------------------------------------

    if text=="A"{

        return Ok(PictureClause{
            signed,
            category:PictureCategory::Alphabetic,
            length:1,
            scale:0,
            usage,
        });

    }

    //--------------------------------------------------
    // PIC A(n)
    //--------------------------------------------------

    if text.starts_with("A("){

        if let Some(end)=text.find(')'){

            let len=&text[2..end];

            if let Ok(length)=len.parse::<usize>(){

                return Ok(PictureClause{
                    signed,
                    category:PictureCategory::Alphabetic,
                    length,
                    scale:0,
                    usage,
                });

            }

        }

    }

    //--------------------------------------------------
    // PIC 9
    //--------------------------------------------------

    if text=="9"{

        return Ok(PictureClause{
            signed,
            category:PictureCategory::Numeric,
            length:1,
            scale:0,
            usage,
        });

    }

    //--------------------------------------------------
    // PIC 9(n)
    //--------------------------------------------------

    if text.starts_with("9("){

        if let Some(end)=text.find(')'){

            let len=&text[2..end];

            if let Ok(length)=len.parse::<usize>(){

                return Ok(PictureClause{
                    signed,
                    category:PictureCategory::Numeric,
                    length,
                    scale:0,
                    usage,
                });

            }

        }

    }

    Err(format!("Unsupported PIC: {}",input))

}
