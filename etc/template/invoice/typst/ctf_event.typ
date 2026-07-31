#set page(
  margin: (top: 0pt, right: 0pt, left: 0pt, bottom: 50pt),
  footer: [
    #pad(x: 30pt)[
      #place(top, line(length: 100%, stroke: 0.5pt + rgb("#e0e0e0")))
      #pad(top: 10pt, right: 15pt)[
        #align(right)[
          #text(
            size: 9pt,
            fill: rgb("#000000"),
            font: "Ubuntu",
            weight: 300,
          )[Page 1 of 1]]]]
  ],
)
#place(
  top,
  rect(
    width: 100%,
    height: 8pt,
    stroke: none,
    fill: gradient.linear(
      rgb("#202328"),
      rgb("#22252b"),
      rgb("#24282e"),
      rgb("#262a30"),
      rgb("#282c33"),
      rgb("#2a2e36"),
      rgb("#282c33"),
      rgb("#262a30"),
      rgb("#24282e"),
      rgb("#22252b"),
      rgb("#202328"),
    ),
  )
)
#pad(x: 30pt)[
  #grid(
    columns: (1fr, auto),
    align: (left + top, right + top),
    [
      #v(60pt)
      #text(
        size: 25pt,
        weight: 400,
        font: "Unbounded",
      )[Invoice]
    ],
    [
      #v(20pt)
      #stack(
        spacing: 0pt,
        image("logo.png", width: 100pt),
        text(
          size: 20pt,
          weight: 300,
          fill: rgb("#343434"),
          font: "Bowlby One"
        )[EXPLOITX]
      )
    ]
  )
]
#v(-30pt)
#set text(
  font: "Ubuntu",
  weight: 400,
  size: 10pt
)
#pad(x: 30pt)[
  #grid(
    columns: (100pt, 1fr),
    row-gutter: 9pt,
    [Invoice number], [EX_ITV2_987654321],
    [Invoice status], [#text(fill: rgb("#008000"), weight: "black")[PAID]],
    [Issued on], [12:23 AM IST, June 30, 2026],
    [Paid on], [12:25 AM IST, June 30, 2026],
  )
]
#v(20pt)
#pad(x: 30pt)[
  #set par(leading: 0.8em)
  #grid(
    columns: (200pt, 1fr),
    gutter: 1cm,
    align(left)[
      #v(0pt)
      *ExploitX, Org.* \
      CIT Chennai, Sarathy Nagar \
      Nandhambakkam Post, Kundrathur \
      Chennai - 600069 \
      Tamil Nadu, India \
      #link("mailto:connect@exploitx.org")[connect\@exploitx.org]
    ],
    align(left)[
      *Bill to* \
      #pad(x: 0pt)[
       Akilesh. A. S \
        Sannathi Street, North Soorankudy, Dharmapuram \
        Near Government Higher Secondary School, Nagercoil \
        Kanyakumari - 629501 \
        Tamil Nadu, India \
        #link("mailto:io@akileshas.dev")[io\@akileshas.dev]
      ]
    ],
  )
]
#v(20pt)
#pad(x: 30pt)[
  #text(
    "Paid ₹150 on 12:25 AM IST, June 30, 2026.",
    weight: "black",
    size: 15pt,
  )
]
#pad(x: 30pt)[
  #text(
    weight: "medium",
  )[
    Successfully processed online via #link("https://www.cashfree.com/")[
      #underline(stroke: 0.5pt+rgb("#1a73e8"))[
        #text(fill: rgb("#1a73e8"))[CashFree]]].
  ]
]
#v(10pt)
#pad(x: 30pt)[

  #table(
    columns: (1.5fr, 1fr, 40pt, 60pt, 50pt, 70pt),
    align: (col, row) => (
      if col == 0 or (row > 1 and col == 1) { left } else { right }
    ),
    stroke: (x, y) => if y == 0 {
      (bottom: 0.6pt + rgb("#000000"))
    },
    table.cell(colspan: 2, inset: (bottom: 10pt))[Description],
    table.cell(inset: (bottom: 10pt))[Qty],
    table.cell(inset: (bottom: 10pt))[Unit price],
    table.cell(inset: (bottom: 10pt))[Tax],
    table.cell(inset: (bottom: 10pt))[Amount],
    table.cell(colspan: 2, inset: (top: 10pt, bottom: 15pt))[
      Registration Fee - Into The Void 2.0 CTF Event \
    ],
    table.cell(inset: (top: 10pt, bottom: 8pt))[1],
    table.cell(inset: (top: 10pt, bottom: 8pt))[₹ 200.00],
    table.cell(inset: (top: 10pt, bottom: 8pt))[0%],
    table.cell(inset: (top: 10pt, bottom: 8pt))[₹ 200.00],

    table.cell(stroke: none)[],
    table.cell(colspan: 4, stroke: (top: 0.5pt + rgb("#e0e0e0"), bottom: 0.5pt + rgb("#e0e0e0")))[Subtotal],
    table.cell(stroke: (top: 0.5pt + rgb("#e0e0e0"), bottom: 0.5pt + rgb("#e0e0e0")))[₹ 200.00],

    table.cell(stroke: none)[],
    table.cell(colspan: 4, stroke: (bottom: 0.5pt + rgb("#e0e0e0")))[Discount \[ Promo: FLAGFOUND \] (-25%)],
    table.cell(stroke: (bottom: 0.5pt + rgb("#e0e0e0")))[\-₹ 50.00],

    table.cell(stroke: none)[],
    table.cell(colspan: 4, stroke: (bottom: 0.5pt + rgb("#e0e0e0")))[Total excluding tax],
    table.cell(stroke: (bottom: 0.5pt + rgb("#e0e0e0")))[₹ 150.00],

    table.cell(stroke: none)[],
    table.cell(colspan: 4, stroke: (bottom: 0.5pt + rgb("#e0e0e0")))[Total tax],
    table.cell(stroke: (bottom: 0.5pt + rgb("#e0e0e0")))[₹ 0.00],

    table.cell(stroke: none)[],
    table.cell(colspan: 4, stroke: (bottom: 0.5pt + rgb("#e0e0e0")))[Total],
    table.cell(stroke: (bottom: 0.5pt + rgb("#e0e0e0")))[₹ 150.00],

    table.cell(stroke: none)[],
    table.cell(colspan: 4, stroke: (bottom: 0.5pt + rgb("#e0e0e0")))[*Amount Paid*],
    table.cell(stroke: (bottom: 0.5pt + rgb("#e0e0e0")))[*₹ 150.00*],

    table.cell(stroke: none)[],
    table.cell(colspan: 4)[*Balance Due*],
    table.cell[*₹ 0.00*]
  )

]
#v(20pt)
#pad(x: 30pt)[
  #text()[
    For any other billing concerns, please email your request to
    #underline(stroke: 0.5pt+rgb("#1a73e8"))[
      #text(fill: rgb("#1a73e8"))[
        #link("mailto:support@exploitx.org")[
          support\@exploitx.org]]].
  ]
]
#v(10pt)
#pad(x: 30pt)[
  #set par(leading: 0.7em)
  A payment of
  #text(
    weight: "medium",
  )[₹150]
  was successfully transferred to the following \
  recipient account:
  #v(0.1em)
  #grid(
    columns: (150pt, 1fr),
    row-gutter: 0.7em,
    [Bank Name], [The Karur Vysya Bank Limited],
    [Account Number], [128XXXXXXXXXX949],
    [Account Holder Name], [Mr. DAMODHARAN J],
    [Reference], [EX_ITV2_987654321],
    [CashFree Transaction ID], [CF_TXN_987654321],
    [Payment Status], [SUCCESS]
  )
]
