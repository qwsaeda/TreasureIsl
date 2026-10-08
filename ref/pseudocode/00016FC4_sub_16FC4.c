/* ea:   0x00016FC4 */
/* name: sub_16FC4 */
/* size: 60 */
int __fastcall sub_16FC4(_DWORD *a1, int a2)
{
  void *v3; // r0
  int result; // r0

  v3 = (void *)sub_17000(a2, *a1 + 4, *a1 + 8, *a1 + 12);
  if ( v3 != nullptr )
    free(v3);
  result = a1[1];
  a1[1] = result - 1;
  return result;
}

