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

    let mut usage = Usage::Display;

    if text.ends_with(" COMP-3") {
        usage = Usage::Comp3;
        text = text.replace(" COMP-3", "");
    }
    else if text.ends_with(" COMP") {
        usage = Usage::Comp;
        text = text.replace(" COMP", "");
    }
    else if text.ends_with(" BINARY") {
        usage = Usage::Binary;
        text = text.replace(" BINARY", "");
    }
    else if text.ends_with(" DISPLAY") {
        usage = Usage::Display;
        text = text.replace(" DISPLAY", "");
    }

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


    //--------------------------------------------------
    // Numeric with decimal
    //--------------------------------------------------

    if text.contains("V") {

        let parts = text.Split("V");

        if(parts.Length -eq 2){}

        let left = parts[0];
        let right = parts[1];

        let mut int_len = 0usize;
        let mut frac_len = 0usize;

        if left.starts_with("9(") {

            if let Some(end)=left.find(')'){

                int_len = left[2..end].parse::<usize>().unwrap_or(0);

            }

        }

        if right.starts_with("9("){

            if let Some(end)=right.find(')'){

                frac_len = right[2..end].parse::<usize>().unwrap_or(0);

            }

        } else {

            frac_len = right.matches('9').count();

        }

        return Ok(PictureClause{

            signed,

            category:PictureCategory::Numeric,

            length:int_len+frac_len,

            scale:frac_len,

            usage,

        });

    }

    Err(format!("Unsupported PIC: {}",input))

}

