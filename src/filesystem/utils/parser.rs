pub fn parse_cgroup_name(data: &str) -> nom::IResult<&str, &str, ()> {
    use nom::Parser as _;
    use nom::branch::*;
    use nom::bytes::complete::*;
    use nom::character::complete::*;
    use nom::combinator::*;
    use nom::multi::*;

    let folder_name_parser = ||
        alt((
            tag("."),
            recognize((

                alt((alpha1::<_, ()>, tag("_"))),
                many0_count(alt((alphanumeric1, tag("_"))))
            ))
        ));

    let name_parser = ||
        recognize(separated_list1(tag("/"), folder_name_parser()));

    name_parser().parse(data)
}

pub fn parse_cgroup_alloc_request(data: &str) -> nom::IResult<&str, crate::manager::Reservation, ()> {
    use nom::Parser as _;
    use nom::character::complete::*;
    use nom::combinator::*;

    map(
        (
            parse_u64,
            space1,
            parse_u64,
        ),
        |(runtime_us, _, period_us)|
            crate::manager::Reservation { runtime_us, period_us }
    ).parse(data)
}

pub fn parse_u64(data: &str) -> nom::IResult<&str, u64, ()> {
    use nom::Parser as _;
    use nom::character::complete::*;
    use nom::combinator::*;

    map_res(
        recognize(digit1),
        |str: &str| str.parse()
    ).parse(data)
}