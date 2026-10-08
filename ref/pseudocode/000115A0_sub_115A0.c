/* ea:   0x000115A0 */
/* name: sub_115A0 */
/* size: 56 */
void *__fastcall sub_115A0(int a1, int a2)
{
  int v2; // r4
  void *result; // r0

  if ( a2 == 0 )
    return nullptr;
  v2 = 12 * a2;
  result = malloc(12 * a2);
  if ( result == nullptr )
    return (void *)sub_115D8(v2);
  return result;
}

