/* ea:   0x00064400 */
/* name: sub_64400 */
/* size: 52 */
void *__fastcall sub_64400(int a1, int a2)
{
  size_t v2; // r4
  void *result; // r0

  if ( a2 == 0 )
    return nullptr;
  v2 = 16 * a2;
  result = malloc(16 * a2);
  if ( result == nullptr )
    return sub_115D8(v2);
  return result;
}

