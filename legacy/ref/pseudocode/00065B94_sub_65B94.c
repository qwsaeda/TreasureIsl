/* ea:   0x00065B94 */
/* name: sub_65B94 */
/* size: 20 */
void *__fastcall sub_65B94(void *result, const void *a2, int a3)
{
  if ( a3 != 0 )
    return memmove(result, a2, 2 * a3);
  return result;
}

