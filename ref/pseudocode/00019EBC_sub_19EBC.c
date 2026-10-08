/* ea:   0x00019EBC */
/* name: sub_19EBC */
/* size: 68 */
int __fastcall sub_19EBC(_DWORD *a1)
{
  int i; // r4
  void *v3; // r0

  for ( i = *a1; i != a1[1]; i += 8 )
  {
    v3 = *(void **)(i + 4);
    if ( v3 == nullptr )
      __debugbreak();
    free(v3);
  }
  return sub_19F00(a1, *a1, a1[1]);
}

