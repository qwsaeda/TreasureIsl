/* ea:   0x0003D258 */
/* name: sub_3D258 */
/* size: 36 */
void __fastcall sub_3D258(void ***a1)
{
  void **v1; // r0
  void **v2; // r4

  v1 = *a1;
  v2 = v1;
  if ( v1 != nullptr )
  {
    sub_16B24(v1);
    operator delete(v2);
  }
}

