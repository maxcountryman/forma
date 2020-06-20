SELECT 
    employee_id, 
    first_name, 
    last_name, 
    salary
FROM employees
ORDER BY 
    salary DESC
OFFSET 0 ROWS
FETCH FIRST 1 ROWS ONLY;
SELECT 
    employee_id, 
    first_name, 
    last_name, 
    salary
FROM employees
ORDER BY 
    salary DESC
OFFSET 5 ROWS
FETCH NEXT 10 PERCENT ROWS WITH TIES;
SELECT 
    employee_id, 
    first_name, 
    last_name, 
    salary
FROM employees
ORDER BY 
    salary DESC
OFFSET 1 ROW
FETCH FIRST ROWS ONLY;
