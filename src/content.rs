pub const HEADER : &str = "<html>
                    <body>
                        <h3> Hello! I see you want to save some amount on your Jio Recharge. I hope this data helps you!</h3>
                        <h4> This data is refreshed every 5 minutes, parsed and presented here. If you have any suggestions or feedback, please feel free to reach out to me at karupal2002@gmail.com </h4>
                        <h4> If you wanna see the code behind this and my other works, click <a href=\"https://navinshrinivas.com\">here</a>. If you wanna see the sorting logic, scroll down!</h4>";
pub const FOOTER : &str = "
        <h4> Sorting Logic </h4>
        <ul>
            <li> First and foremost, plans with valid calling are shown first </li>
            <li> Then I sort by price_per_day followed by price_per_gb and then price and then by validity </li>
            <li> Else simply by name </li>
        </ul>
            <p> This sorting logic is designed for my usecase, that is to have some data every day (not a lot) and have calling all year round. </p> 
            <p> India is in a stage where we need data everyday when we are outside our home (for UPI and what not). At home I have wifi, so I don't need a lot of data. </p>
                    </body>
                    </html>";
