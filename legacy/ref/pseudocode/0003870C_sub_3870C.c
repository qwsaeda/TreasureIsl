/* ea:   0x0003870C */
/* name: sub_3870C */
/* size: 56 */
void *__fastcall sub_3870C(int a1, int a2)
{
  size_t v2; // r4
  void *result; // r0

  if ( a2 == 0 )
    return nullptr;
  v2 = 12 * a2;
  result = malloc(12 * a2);
  if ( result == nullptr )
    return sub_115D8(v2);
  return result;
}

