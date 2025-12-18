const Joi = require('joi');

// Item validation schema
const itemSchema = Joi.object({
    name: Joi.string().required(),
    quantity: Joi.number().optional(),
    completed: Joi.boolean().optional().default(false),
});

module.exports = itemSchema;
