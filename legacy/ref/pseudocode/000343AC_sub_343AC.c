/* ea:   0x000343AC */
/* name: sub_343AC */
/* size: 56 */
void *__fastcall sub_343AC(int a1, int a2)
{
  size_t v2; // r4
  void *result; // r0

  if ( a2 == 0 )
    return nullptr;
  v2 = 24 * a2;
  result = malloc(24 * a2);
  if ( result == nullptr )
    return sub_115D8(v2);
  return result;
}

