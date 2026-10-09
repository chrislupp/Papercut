// Internal template. All user text is passed as quoted strings, never evaluated.
#let plain(value) = value.split("\n\n").map(part => text(part)).join(parbreak())
#let filename = state("papercut-filename", "")
#let hf-text(value, date) = context {
  let name = filename.get()
  // A file beginning on this page supplies its header as well as its footer.
  let files = query(metadata).filter(m => type(m.value) == dictionary and "filename" in m.value)
  let current = files.filter(m => m.location().page() == here().page())
  if current.len() > 0 { name = current.first().value.filename }
  plain(value.replace("{page}", str(counter(page).get().first()))
    .replace("{total}", str(counter(page).final().first()))
    .replace("{filename}", name).replace("{date}", date))
}
#let source-line(spans, number, numbers, separator, borders, background, foreground, number-color, wrap, indent, line-height) = layout(size => {
  let char-width = measure(text("0")).width
  let gutter = if numbers { 4 * char-width } else { 0pt }
  let capacity = calc.max(1, calc.floor((size.width - gutter - 8pt) / char-width))
  let continuation = calc.min(indent, capacity - 1)
  let rows = ()
  let row = []
  let used = 0
  for span in spans {
    let chars = span.text.clusters()
    while chars.len() > 0 {
      let count = if wrap { calc.min(chars.len(), capacity - used) } else { chars.len() }
      let part = text(fill: rgb(span.color), weight: if span.bold { "bold" } else { "regular" }, style: if span.italic { "italic" } else { "normal" }, chars.slice(0, count).join())
      if span.underline { part = underline(part) }
      row += part
      used += count
      chars = chars.slice(count)
      if wrap and used >= capacity {
        rows.push(row)
        row = h(continuation * char-width)
        used = continuation
      }
    }
  }
  if used > continuation or rows.len() == 0 { rows.push(row) }
  for (i, row) in rows.enumerate() {
    block(width: 100%, height: line-height, above: 0pt, below: 0pt, breakable: false, fill: rgb(background),
      stroke: (left: if borders { 0.4pt + luma(190) } else { none }, right: if borders { 0.4pt + luma(190) } else { none }),
      inset: (x: 4pt),
      grid(columns: (gutter, 1fr), column-gutter: 0pt,
        if numbers { block(width: 100%, inset: (right: char-width), stroke: (right: if separator { 0.4pt + luma(190) } else { none }), align(right, text(fill: rgb(number-color), if i == 0 { str(number) } else { "" }))) },
        box(width: 100%, height: line-height, clip: true,
          box(width: calc.max(1pt, measure(row).width), text(fill: rgb(foreground), row)))))
  }
})
