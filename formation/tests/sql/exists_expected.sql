select * from t1 where exists (select id from t2 where created_at > "2020");
