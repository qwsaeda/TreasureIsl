/* ea:   0x0005022C */
/* name: sub_5022C */
/* size: 52 */
void *__fastcall sub_5022C(int a1, int a2)
{
  size_t v2; // r4
  void *result; // r0

  if ( a2 == 0 )
    return nullptr;
  v2 = 4 * a2;
  result = malloc(4 * a2);
  if ( result == nullptr )
    return sub_115D8(v2);
  return result;
}

