(select * from t1
union
select * from t2)
intersect
(select * from t3)
order by c1;
