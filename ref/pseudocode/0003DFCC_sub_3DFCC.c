/* ea:   0x0003DFCC */
/* name: sub_3DFCC */
/* size: 36 */
void __fastcall sub_3DFCC(void **a1)
{
  void *v1; // r0
  void *v2; // r4

  v1 = *a1;
  v2 = v1;
  if ( v1 != nullptr )
  {
    sub_590A4(v1);
    operator delete(v2);
  }
}

