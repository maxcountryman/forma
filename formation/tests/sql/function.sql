SELECT
  c1, 
  LAST_VALUE(c1) OVER(ORDER BY c1 ROWS BETWEEN UNBOUNDED PRECEDING AND UNBOUNDED FOLLOWING) AS "last_val"
FROM t1;
SELECT order_id, customer_id,
       FIRST_VALUE(order_id) OVER(PARTITION BY customer_id ORDER BY order_id) AS first_order_id
FROM orders
WHERE customer_id IN (1,2)
