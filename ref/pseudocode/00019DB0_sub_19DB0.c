/* ea:   0x00019DB0 */
/* name: sub_19DB0 */
/* size: 52 */
void *__fastcall sub_19DB0(int a1, int a2)
{
  size_t v2; // r4
  void *result; // r0

  if ( a2 == 0 )
    return nullptr;
  v2 = 8 * a2;
  result = malloc(8 * a2);
  if ( result == nullptr )
    return sub_115D8(v2);
  return result;
}

