/* ea:   0x0001B034 */
/* name: sub_1B034 */
/* size: 56 */
void *__fastcall sub_1B034(int a1, int a2)
{
  size_t v2; // r4
  void *result; // r0

  if ( a2 == 0 )
    return nullptr;
  v2 = 36 * a2;
  result = malloc(36 * a2);
  if ( result == nullptr )
    return sub_115D8(v2);
  return result;
}

