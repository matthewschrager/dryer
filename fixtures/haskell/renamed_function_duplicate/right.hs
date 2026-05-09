module Invoices where

buildInvoiceRows invoices =
  let active = filter (\invoice -> invoice.amount > 0) invoices
      rows = map (\invoice -> (invoice.id, invoice.amount, invoice.currency)) active
      total = foldl (\acc row -> acc + row.amount) 0 active
  in map
      (\row ->
        if row.amount > total
          then (row.id, row.currency, "large")
          else (row.id, row.currency, "standard"))
      active
