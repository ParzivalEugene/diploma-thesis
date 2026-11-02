#let line-gap = 0.725em
#let body-size = 14pt

#let report(body) = {
  set document(title: "Отчёт по НИР", author: "Мичков Е.В.")
  set page(
    paper: "a4",
    margin: (left: 30mm, right: 15mm, top: 20mm, bottom: 20mm),
    numbering: none,
  )
  set text(
    font: "Times New Roman",
    size: body-size,
    lang: "ru",
    fill: black,
    overhang: false,
    top-edge: 1em,
    bottom-edge: 0em,
  )
  set par(
    first-line-indent: (amount: 1.25cm, all: true),
    justify: true,
    leading: line-gap,
    spacing: line-gap,
  )
  show math.equation: set text(font: (
    "STIX Two Math",
    "New Computer Modern Math",
  ))
  set math.equation(numbering: "(1)", supplement: none)
  show math.equation.where(block: true): set block(above: 1.8em, below: 1.8em)

  set list(marker: [--], indent: 1.25cm, spacing: line-gap)
  set enum(numbering: "1)", indent: 1.25cm, spacing: line-gap)

  set heading(numbering: "1.1")
  show heading: it => {
    set text(size: body-size, weight: "bold", hyphenate: false)
    set par(justify: false, first-line-indent: 0pt)
    if it.level == 1 { pagebreak(weak: true) }
    if it.numbering == none {
      block(below: 1.2em, width: 100%, align(center, upper(it.body)))
    } else {
      block(
        above: 1.6em,
        below: 1.2em,
        pad(left: 1.25cm, [#counter(heading).display(it.numbering) #it.body]),
      )
    }
  }

  show figure.where(kind: image): set figure(supplement: "Рисунок")
  show figure.where(kind: table): set figure(supplement: "Таблица")
  show figure.where(kind: table): set figure.caption(position: top)
  show figure.where(kind: table): set block(breakable: true)
  set figure.caption(separator: [ – ])
  show figure.caption: set text(hyphenate: false)
  show figure.caption: set par(
    leading: 0.15em,
    justify: false,
    first-line-indent: 0pt,
  )
  show figure.caption.where(position: top): set align(left)
  show figure: set block(above: 1.4em, below: 1.4em)
  show table: set par(leading: 0.3em, justify: false, first-line-indent: 0pt)

  show ref: it => {
    let target = it.element
    if target != none and target.func() == figure {
      link(target.location(), numbering(target.numbering, ..target.counter.at(
        target.location(),
      )))
    } else {
      it
    }
  }
  show outline.entry: set text(weight: "regular")
  body
}

#let section(title) = heading(level: 1, numbering: none, title)

#let gost-table(
  columns: auto,
  align: auto,
  caption: none,
  label-name: none,
  header: (),
  ..cells,
) = {
  let continuation = context {
    let first = query(selector(label(label-name))).first().location().page()
    if here().page() != first {
      pad(bottom: 6pt, std.align(
        left,
      )[Продолжение таблицы #counter(figure.where(kind: table)).display()])
    }
  }
  let column-count = if type(columns) == array { columns.len() } else {
    columns
  }
  [#figure(
      caption: caption,
      table(
        columns: columns,
        align: align,
        table.header(
          table.cell(
            colspan: column-count,
            stroke: none,
            inset: 0pt,
            continuation,
          ),
          ..header,
        ),
        ..cells,
      ),
    ) #label(label-name)]
}

#let title-page(
  group: "",
  topic: "",
  semester: "",
  year: "",
  student: "",
  supervisor: "",
  city-year: "",
) = {
  set page(
    margin: (left: 20mm, right: 10mm, top: 14mm, bottom: 14mm),
    numbering: none,
  )
  set par(
    first-line-indent: 0pt,
    justify: false,
    leading: 0.5em,
    spacing: 0.5em,
  )
  align(center)[НИТУ «МИСИС»]
  v(1.5em)
  [Институт ИТКН \ Кафедра инженерной кибернетики \ Направление подготовки: 01.03.04 прикладная математика \ Квалификация (степень): бакалавр \ Группа: #group]
  v(4em)
  align(center)[
    #text(weight: "bold")[ОТЧЕТ \ ПО НАУЧНО-ИССЛЕДОВАТЕЛЬСКОЙ РАБОТЕ]
    #v(1em)
    на тему: «#topic»
    #v(1em)
    #semester \
    #year уч. год.
  ]
  v(5em)
  let signature(role, hint, name) = grid(
    columns: (auto, 1fr, auto),
    column-gutter: 8pt,
    role, [#box(width: 3.5cm, repeat[\_]) / #name /], [],
  )
  signature([Студент], [], student)
  v(1.5em)
  signature([Руководитель НИР], [], supervisor)
  v(3em)
  [Оценка: #box(width: 9cm, repeat[\_])]
  v(1em)
  [Дата защиты: #box(width: 8.2cm, repeat[\_])]
  v(3em)
  [Утвердил: \ Председатель комиссии #box(width: 3.5cm, repeat[\_]) / #box(width: 4cm, repeat[\_]) /]
  v(1fr)
  align(center)[Москва #city-year]
  pagebreak()
}

#let start-numbering() = {
  set page(numbering: "1", number-align: center)
}
