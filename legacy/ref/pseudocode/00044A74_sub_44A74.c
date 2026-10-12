/* ea:   0x00044A74 */
/* name: sub_44A74 */
/* size: 56 */
void *__fastcall sub_44A74(int a1, int a2)
{
  size_t v2; // r4
  void *result; // r0

  if ( a2 == 0 )
    return nullptr;
  v2 = 60 * a2;
  result = malloc(60 * a2);
  if ( result == nullptr )
    return sub_115D8(v2);
  return result;
}

