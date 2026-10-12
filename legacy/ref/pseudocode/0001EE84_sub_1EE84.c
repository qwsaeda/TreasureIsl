/* ea:   0x0001EE84 */
/* name: sub_1EE84 */
/* size: 56 */
void *__fastcall sub_1EE84(int a1, int a2)
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

