select
  c1,
  last_value(c1) over(
    order by c1
    rows between unbounded preceding and unbounded following
  ) as "last_val"
from
  t1;
select
  order_id,
  customer_id,
  first_value(order_id) over(partition by customer_id order by order_id) as first_order_id
from
  orders
where
  customer_id in (1, 2);
