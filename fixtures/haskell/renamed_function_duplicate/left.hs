module Orders where

buildOrderRows orders =
  let active = filter (\order -> order.amount > 0) orders
      rows = map (\order -> (order.id, order.amount, order.currency)) active
      total = foldl (\acc row -> acc + row.amount) 0 active
  in map
      (\row ->
        if row.amount > total
          then (row.id, row.currency, "large")
          else (row.id, row.currency, "standard"))
      active
