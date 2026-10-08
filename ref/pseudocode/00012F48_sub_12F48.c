/* ea:   0x00012F48 */
/* name: sub_12F48 */
/* size: 20 */
void __fastcall sub_12F48(void **a1)
{
  void *v1; // r0

  v1 = *a1;
  if ( v1 != nullptr )
    free(v1);
}

