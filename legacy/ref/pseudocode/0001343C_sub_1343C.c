/* ea:   0x0001343C */
/* name: sub_1343C */
/* size: 52 */
void *__fastcall sub_1343C(int a1, int a2)
{
  size_t v2; // r4
  void *result; // r0

  if ( a2 == 0 )
    return nullptr;
  v2 = 2 * a2;
  result = malloc(2 * a2);
  if ( result == nullptr )
    return sub_115D8(v2);
  return result;
}

