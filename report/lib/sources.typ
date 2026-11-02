#let catalog = yaml("/report/sources.yaml")

#let cited-in-order() = {
  let seen = ()
  for entry in query(<nir-source>) {
    for key in entry.value {
      if key not in seen { seen.push(key) }
    }
  }
  seen
}

#let src(..keys) = {
  let names = keys.pos()
  for name in names {
    assert(name in catalog, message: "источника нет в sources.yaml: " + name)
  }
  [#metadata(names)<nir-source>]
  context {
    let order = cited-in-order()
    let numbers = names
      .map(name => order.position(item => item == name) + 1)
      .sorted()
    "[" + numbers.map(str).join(", ") + "]"
  }
}

#let source-list() = context {
  let order = cited-in-order()
  set par(first-line-indent: 0pt)
  set enum(numbering: "1.", indent: 0pt)
  enum(tight: false, ..order.map(name => eval(
    catalog.at(name).text,
    mode: "markup",
  )))
}

#let foreign-print-count() = {
  let foreign = cited-in-order().filter(name => {
    let source = catalog.at(name)
    source.lang != "ru" and source.kind == "print"
  })
  foreign.len()
}
