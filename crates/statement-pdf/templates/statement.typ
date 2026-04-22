#let data = json("statement.json")

#set document(
  title: "Заявление " + data.statement_number,
)

#set page(
  paper: "a4",
  margin: (
    top: 20mm,
    right: 15mm,
    bottom: 20mm,
    left: 30mm,
  ),
)

#set text(
  font: "Times New Roman",
  size: 12pt,
  lang: "ru",
)

#set par(
  justify: true,
  leading: 0.65em,
  first-line-indent: 1.25cm,
)

#let has-signature = data.signature != none
#let has-signature-image = has-signature and data.signature.image != none
#let signer-name = if has-signature and data.signature.signer_name != none {
  data.signature.signer_name
} else {
  data.applicant.full_name
}

#align(right)[
  #block(width: 82mm)[
    #data.recipient.position \
    #data.recipient.company_name \
    #data.recipient.full_name

    #v(10pt)

    от #data.applicant.full_name \
    #data.applicant.department
  ]
]

#v(24pt)

#align(center)[
  #strong[ЗАЯВЛЕНИЕ]
]

#v(16pt)

#data.body

#v(30pt)

#grid(
  columns: (1fr, 54mm, 1.35fr),
  gutter: 8mm,
  align: horizon,
  [#data.date],
  [
    #box(width: 54mm, height: 22mm)[
      #place(
        dx: 10mm,
        dy: -15mm,
        image(data.stamp.path, width: 34mm),
      )
      #if has-signature-image {
        image(data.signature.image.path, width: 48mm)
      } else {
        line(length: 48mm)
      }
    ]
  ],
  [#signer-name],
)
