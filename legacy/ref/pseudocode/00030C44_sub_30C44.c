/* ea:   0x00030C44 */
/* name: sub_30C44 */
/* size: 60 */
int __fastcall sub_30C44(_DWORD *a1, char *a2)
{
  char *v3; // r0
  int result; // r0

  v3 = sub_17000(a2, (char **)(*a1 + 4), (char **)(*a1 + 8), (char **)(*a1 + 12));
  if ( v3 != nullptr )
    free(v3);
  result = a1[1];
  a1[1] = result - 1;
  return result;
}

