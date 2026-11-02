#import "lib/template.typ": report, section, title-page
#import "lib/sources.typ": cited-in-order, foreign-print-count, source-list, src

#show: report

#title-page(
  group: "БПМ-23-РК-1",
  topic: "Разработка и экспериментальное исследование методов предобработки графа для поиска маршрута на дорожных сетях с зависящими от времени весами рёбер",
  semester: "7-й семестр",
  year: "2026/2027",
  student: "Мичков Е.В.",
  supervisor: "ассистент Вишневский М.Р.",
  city-year: "2026",
)

#set page(numbering: "1", number-align: center)

#outline(title: [СОДЕРЖАНИЕ], depth: 3, indent: 1em)

#include "chapters/00-abbreviations.typ"
#include "chapters/01-introduction.typ"
#include "chapters/02-review.typ"
#include "chapters/03-problem-statement.typ"
#include "chapters/04-mathematical-statement.typ"
#include "chapters/05-software.typ"
#include "chapters/06-experiments.typ"
#include "chapters/07-conclusions.typ"

#section[Список использованных источников]
#source-list()
#metadata(none) <report-end>

#let report-facts() = {
  let page-of(target) = query(target).first().location().page()
  let figures = query(figure)
  let referenced = query(ref).map(item => str(item.target))
  (
    review_start: page-of(<review-start>),
    review_end: page-of(<review-end>),
    report_end: page-of(<report-end>),
    cited: cited-in-order().len(),
    foreign_print: foreign-print-count(),
    unreferenced_figures: figures
      .filter(item => item.has("label") and str(item.label) not in referenced)
      .map(item => str(item.label)),
  )
}
#context [#metadata(report-facts()) <report-facts>]
