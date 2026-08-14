-- Insert default super admin
-- Password: Admin@123 (bcrypt hashed, verified)
INSERT INTO users (email, password_hash, full_name, role, permissions)
VALUES (
    'admin@pcfi.com.np',
    '$2b$12$vUfBlu8aPqY/7e0IRPjcWudZrKURu80VSZ3EabT4OFhFgSlTwUceO',
    'System Administrator',
    'superadmin',
    '{"can_manage_users": true, "can_manage_products": true, "can_manage_gallery": true, "can_manage_content": true}'
);

-- Insert default hero section
INSERT INTO hero_sections (heading, subheading, description, primary_cta_text, primary_cta_link, secondary_cta_text, secondary_cta_link, is_active)
VALUES (
    'Welcome to Progressive Cattle Fodder',
    'Nepal''s Trusted Livestock Feed Manufacturer',
    'We provide premium-quality corn silage and feed solutions for your livestock — fresh, nutritious, and sustainable. With over four years of expertise, we deliver scientifically prepared silage designed for maximum freshness, digestibility, and farm efficiency.',
    'Contact Us Now',
    '/contact',
    'Explore More',
    '/products',
    true
);

-- Insert content blocks
INSERT INTO content_blocks (key, title, content, content_type, metadata, is_published) VALUES
(
    'about_company',
    'About PCFI',
    'PDAS Pvt. Ltd. is a trusted provider of agricultural solutions in Nepal. With over eight years of expertise, we are dedicated to enhancing productivity and sustainability for farmers across the country. Our products are scientifically formulated to meet the healthy habitat and production needs of livestock, ensuring optimal health and performance. We are committed to supporting the agricultural community with innovative solutions that drive growth and prosperity.',
    'text',
    '{}',
    true
),
(
    'chairman_message',
    'Message from our Chairman',
    '"At PDAS, we are driven by a mission to empower farmers with sustainable, high-quality dairy and agro solutions. Our vision is not only to provide machinery and reliable agro and dairy products but also to ensure if the products and parts gets available in time for reliable and constant production to support the prosperity of every livestock owner we serve." — Mr. Gopal Thapa',
    'text',
    '{"author": "Mr. Gopal Thapa", "title": "Chairman"}',
    true
),
(
    'mission',
    'Our Mission',
    'We are committed to empowering farmers and livestock owners with innovative, sustainable, and high-quality production solutions. Our mission and vision drive us to set new standards in Nepal''s agricultural sector.',
    'text',
    '{}',
    true
),
(
    'vision',
    'Our Vision',
    'To be Nepal''s most trusted and innovative livestock solution provider, supporting agricultural prosperity across every region of the country through science-backed solutions and farmer-first values.',
    'text',
    '{}',
    true
),
(
    'contact_info',
    'Contact Information',
    'Get in touch with Progressive Dairy and Agro Solutions',
    'json',
    '{
        "phone": "+977-9802012000",
        "email": "progressive2077@gmail.com",
        "address": "Hetauda-10, Makwanpur, Nepal",
        "social": {
            "facebook": "https://facebook.com/pcfi",
            "instagram": "https://instagram.com/pcfi",
            "youtube": "https://youtube.com/pcfi",
            "linkedin": "https://linkedin.com/company/pcfi",
            "tiktok": "https://tiktok.com/@pcfi",
            "twitter": "https://twitter.com/pcfi"
        },
        "tagline": "Happy Cow, Happy Farmers!"
    }',
    true
);

-- Insert sample products
WITH admin_user AS (SELECT id FROM users WHERE email = 'admin@pcfi.com.np' LIMIT 1)
INSERT INTO products (name, slug, description, short_description, category, is_published, sort_order, features, created_by)
SELECT
    'Agro Machinery',
    'agro-machinery',
    'PCFI Pvt. Ltd. is a trusted manufacturer of high-quality agro machinery, dedicated to improving agricultural productivity and supporting sustainable farming practices. With over eight years of expertise, we deliver innovative equipment designed for maximum efficiency, durability, and performance. Our machinery is engineered to meet the diverse needs of farmers across Nepal, ensuring reliable operation and long-term value.',
    'High-quality agro machinery for improved agricultural productivity.',
    'Agro Machinery',
    true,
    1,
    '["Premium fermentation process", "Extended shelf life up to 18 months", "High nutritional value", "Suitable for all livestock", "Modern wrapping technology"]'::jsonb,
    id
FROM admin_user;

WITH admin_user AS (SELECT id FROM users WHERE email = 'admin@pcfi.com.np' LIMIT 1)
INSERT INTO products (name, slug, description, short_description, category, is_published, sort_order, features, created_by)
SELECT
    'Mash Cattle Feed',
    'mash-cattle-feed',
    'Mash Cattle Feed is a complete, balanced formulation created to meet the nutritional needs of cattle at every growth stage. The mash form ensures fast digestion and smooth mixing with other feed ingredients, making it ideal for both small and large farms. Our formulation is developed by livestock nutrition experts and contains essential vitamins, minerals, and protein sources to promote healthy growth, improved milk production, and better reproductive performance.',
    'Complete, balanced mash feed formulation for all cattle stages.',
    'Dairy Solutions',
    true,
    2,
    '["Complete nutritional profile", "Fast digestion", "Promotes milk production", "All growth stages", "Expert formulated"]'::jsonb,
    id
FROM admin_user;

WITH admin_user AS (SELECT id FROM users WHERE email = 'admin@pcfi.com.np' LIMIT 1)
INSERT INTO products (name, slug, description, short_description, category, is_published, sort_order, features, created_by)
SELECT
    'Corn Silage',
    'corn-silage',
    'Our Corn Silage is produced from freshly harvested corn crops at the optimal stage of maturity, ensuring maximum energy content and palatability. The silage undergoes a controlled anaerobic fermentation process that preserves nutrients, improves digestibility, and reduces feed waste. It is an excellent source of energy for high-producing dairy cows and beef cattle, providing consistent quality throughout the year regardless of seasonal variations.',
    'Energy-rich corn silage from freshly harvested crops.',
    'Core Silage',
    true,
    3,
    '["High energy content", "Year-round availability", "Anaerobic fermentation", "Maximum palatability", "Reduces feed waste"]'::jsonb,
    id
FROM admin_user;

-- Insert sample gallery items
WITH admin_user AS (SELECT id FROM users WHERE email = 'admin@pcfi.com.np' LIMIT 1)
INSERT INTO gallery_items (title, description, image_url, category, sort_order, is_published, created_by)
SELECT 'Corn Silage Process', 'Our state-of-the-art corn silage production process', 'https://images.unsplash.com/photo-1500595046743-cd271d694d30?w=800', 'Production', 1, true, id
FROM admin_user;

WITH admin_user AS (SELECT id FROM users WHERE email = 'admin@pcfi.com.np' LIMIT 1)
INSERT INTO gallery_items (title, description, image_url, category, sort_order, is_published, created_by)
SELECT 'Cattle Feeding', 'Happy and healthy cattle enjoying our premium feed', 'https://images.unsplash.com/photo-1546445317-29f4545e9d53?w=800', 'Farm', 2, true, id
FROM admin_user;

WITH admin_user AS (SELECT id FROM users WHERE email = 'admin@pcfi.com.np' LIMIT 1)
INSERT INTO gallery_items (title, description, image_url, category, sort_order, is_published, created_by)
SELECT 'Bale Silage Storage', 'Properly wrapped and stored bale silage', 'https://images.unsplash.com/photo-1625246333195-78d9c38ad449?w=800', 'Production', 3, true, id
FROM admin_user;

WITH admin_user AS (SELECT id FROM users WHERE email = 'admin@pcfi.com.np' LIMIT 1)
INSERT INTO gallery_items (title, description, image_url, category, sort_order, is_published, created_by)
SELECT 'Mash Feed Processing', 'Our mash cattle feed production facility', 'https://images.unsplash.com/photo-1574943320219-553eb213f72d?w=800', 'Production', 4, true, id
FROM admin_user;
